use anyhow::Context;
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::UdpSocketMessage,
    message::{Request, Response},
    pw::{get_pw_params, PwContext},
};
use pipewire::{MainLoop, Signal};
use std::{
    io::{Read, Write},
    mem::ManuallyDrop,
    net::UdpSocket,
    os::unix::net::UnixStream,
    sync::Arc,
};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let registry =
        tracing_subscriber::registry().with(tracing_subscriber::fmt::layer().with_target(false));
    match tracing_journald::layer() {
        Ok(subscriber) => {
            registry.with(subscriber).init();
        }
        Err(e) => {
            registry.init();
            tracing::error!("couldn't connect to journald: {}", e);
        }
    }

    pipewire::init();

    let main_loop = MainLoop::new()?;
    let context = pipewire::Context::new(&main_loop)?;
    let core = context.connect(None)?;
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").context("couldn't bind to address")?);
    socket.set_nonblocking(true)?;
    let ctx = PwContext {
        socket: Arc::clone(&socket),
    };

    let stream = omic::pw::create_stream(&core)?;
    let _s = ManuallyDrop::new(omic::pw::register_callbacks(&stream, ctx)?);
    let unix_socket = omic::socket::bind()?;

    let _io = main_loop.add_io(unix_socket, IoFlags::IN, move |unix_socket| {
        let closure = |stream: &mut UnixStream| -> Result<(), anyhow::Error> {
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes)?;
            let message: Request = bincode::deserialize(&bytes)?;

            match message {
                Request::Connect { address, port } => {
                    tracing::info!("attempting to connect to {}:{}", address, port);
                    let addr = format!("{}:{}", address, port);

                    socket.connect(addr)?;
                    socket.send(&[UdpSocketMessage::Connect as u8])?;
                    tracing::info!("connection established, connect byte sent");
                }
                Request::Disconnect => {
                    tracing::info!("sending disconnect signal");
                    socket.send(&[UdpSocketMessage::Disconnect as u8])?;
                }
                // TODO: add current state query
                Request::Query => todo!(),
            }

            Ok(())
        };

        if let Ok((mut stream, _)) = unix_socket.accept() {
            let _ = match closure(&mut stream) {
                Ok(_) => stream.write_all(&bincode::serialize(&Response::Ok).unwrap()),
                Err(e) => {
                    let error = format!("error occurred reading from unix socket: {}", e);
                    tracing::error!(error);
                    stream.write_all(&bincode::serialize(&Response::Error(error)).unwrap())
                }
            };
        }
    });

    // TODO: ???
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGINT, move || {
        match omic::socket::disconnect() {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("error disconnecting from socket: {}", e)
            }
        };

        tracing::info!("omicd: disconnected");

        if let Some(main_loop) = main_loop_weak.upgrade() {
            main_loop.quit();
        }
    });

    stream.connect(
        Direction::Output,
        None,
        pipewire::stream::StreamFlags::MAP_BUFFERS | pipewire::stream::StreamFlags::RT_PROCESS,
        &mut get_pw_params(),
    )?;

    main_loop.run();

    Ok(())
}
