use lazy_static::lazy_static;
use libspa::{pod::Pod, sys::SPA_AUDIO_FORMAT_S16_LE};
use opus::Decoder;
use pipewire::{
    buffer::Buffer,
    properties,
    stream::{Stream, StreamListener, StreamRef},
};
use std::{io::Cursor, net::UdpSocket, sync::Arc};

pub struct PipewireContext {
    pub socket: Arc<UdpSocket>,
}

lazy_static! {
    static ref POD_VALUES: Vec<u8> = pipewire::spa::pod::serialize::PodSerializer::serialize(
        Cursor::new(Vec::new()),
        &pipewire::spa::pod::Value::Object(pipewire::spa::pod::object!(
            pipewire::spa::utils::SpaTypes::ObjectParamFormat,
            pipewire::spa::param::ParamType::EnumFormat,
            pipewire::spa::pod::property!(
                pipewire::spa::format::FormatProperties::MediaType,
                Id,
                pipewire::spa::format::MediaType::Audio
            ),
            pipewire::spa::pod::property!(
                pipewire::spa::format::FormatProperties::MediaSubtype,
                Id,
                pipewire::spa::format::MediaSubtype::Raw
            ),
            pipewire::spa::pod::property!(
                pipewire::spa::format::FormatProperties::AudioChannels,
                Int,
                1
            ),
            pipewire::spa::pod::property!(
                pipewire::spa::format::FormatProperties::AudioFormat,
                Id,
                pipewire::spa::format::FormatProperties::from_raw(SPA_AUDIO_FORMAT_S16_LE)
            ),
            pipewire::spa::pod::property!(
                pipewire::spa::format::FormatProperties::AudioRate,
                Int,
                48000
            ),
        )),
    )
    .unwrap()
    .0
    .into_inner();
}

pub fn get_pw_params<'a>() -> Vec<&'a Pod> {
    [Pod::from_bytes(&POD_VALUES).unwrap()].to_vec()
}

pub fn create_stream(core: &pipewire::Core) -> Result<Stream, anyhow::Error> {
    Ok(Stream::new(
        core,
        "omic",
        properties! {
            *pipewire::keys::MEDIA_CLASS => "Audio/Source",
            *pipewire::keys::NODE_NAME => "omic",
            *pipewire::keys::AUDIO_CHANNELS => "1",
            "node.channel-names" => "1"
        },
    )?)
}

pub fn register_callbacks(
    stream: &Stream,
    udp: CallbackData,
) -> Result<StreamListener<CallbackData>, anyhow::Error> {
    Ok(stream
        .add_local_listener_with_user_data::<CallbackData>(udp)
        .process(process_callback)
        .register()?)
}

// WOMic packets have unrelated information in the first 11 bytes.
const WOMIC_OFFSET: usize = 11;
static mut BUF: [u8; 3000] = [0; 3000];
static mut OPUS_OUTPUT: [i16; 1000] = [0; 1000];

pub struct CallbackData {
    pub udp: UdpSocket,
    pub decoder: Decoder,
}

fn process_callback(stream: &StreamRef, data: &mut CallbackData) {
    // Unsafe block to make use of simple buffer copying.
    unsafe {
        let udp = &data.udp;
        let decoder = &mut data.decoder;
        if let Ok(usz) = udp.recv(&mut BUF) {
            let mut buffer = stream.dequeue_buffer().unwrap();
            let data = buffer.datas_mut().first_mut().unwrap();
            let stride = std::mem::size_of::<i16>();
            let chunk = data.chunk_mut();
            let size = decoder
                .decode(&BUF[WOMIC_OFFSET..usz], &mut OPUS_OUTPUT, false)
                .unwrap();
            *chunk.offset_mut() = 0;
            *chunk.stride_mut() = stride as i32;
            // size = amount of i16s, convert to total bytes.
            *chunk.size_mut() = (size * 2) as u32;
            let data = data.data().unwrap();
            // memcpy BUF2 to pipewire buffer
            std::ptr::copy_nonoverlapping(
                OPUS_OUTPUT.as_ptr(),
                data.as_mut_ptr().cast::<i16>(),
                size,
            );
        };
    }
}
