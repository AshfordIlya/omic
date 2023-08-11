use lazy_static::lazy_static;
use libspa::{pod::Pod, sys::SPA_AUDIO_FORMAT_S16_LE};
use pipewire::{
    buffer::Buffer,
    properties,
    stream::{Stream, StreamListener, StreamRef},
};
use std::{io::Cursor, net::UdpSocket, sync::Arc};

const BUFFER_SIZE: i32 = 96 * 8;

pub struct PwContext {
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
        &core,
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
    ctx: PwContext,
) -> Result<StreamListener<PwContext>, anyhow::Error> {
    Ok(stream
        .add_local_listener_with_user_data::<PwContext>(ctx)
        .process(
            |s: &StreamRef, ctx: &mut PwContext| match s.dequeue_buffer() {
                Some(mut buffer) => process_callback(&mut buffer, ctx),
                None => tracing::warn!("out of buffer"),
            },
        )
        .register()?)
}

fn process_callback(buffer: &mut Buffer, ctx: &mut PwContext) {
    let data = buffer.datas_mut().first_mut().unwrap();
    let stride = std::mem::size_of::<i16>() * 1;
    let chunk = data.chunk_mut();

    *chunk.offset_mut() = 0;
    *chunk.stride_mut() = stride as i32;
    *chunk.size_mut() = BUFFER_SIZE as u32;

    let data = data.data().unwrap();
    match ctx.socket.recv(data) {
        Ok(_len) => {}
        Err(_) => {
            data.fill(0);
        }
    };
}
