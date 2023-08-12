use anyhow::Context;
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::{UdpSocketMessage, MAX_MESSAGE_SIZE},
    message::{Request, Response},
    pipewire::{get_pw_params, PipewireContext},
};
use pipewire::{MainLoop, Signal, WeakMainLoop};
use std::{mem::ManuallyDrop, net::UdpSocket, sync::Arc};
use tracing_subscriber::prelude::*;
use uds::UnixSeqpacketConn;

fn main() -> Result<(), anyhow::Error> {
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
    let exit = |main_loop_weak: &WeakMainLoop| {
        match omic::socket::disconnect() {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("error disconnecting from socket: {}", e)
            }
        };

        tracing::info!("disconnected");
        if let Some(main_loop) = main_loop_weak.upgrade() {
            main_loop.quit();
        }
        tracing::info!("main loop finished");
    };

    // this has to be done before passing to context
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGINT, move || exit(&main_loop_weak));
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGTERM, move || exit(&main_loop_weak));

    let context = pipewire::Context::new(&main_loop)?;
    let core = context.connect(None)?;
    let udp_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").context("couldn't bind to address")?);
    udp_socket.set_nonblocking(true)?;
    let ctx = PipewireContext {
        socket: Arc::clone(&udp_socket),
    };

    let stream = omic::pipewire::create_stream(&core)?;
    let _s = ManuallyDrop::new(omic::pipewire::register_callbacks(&stream, ctx)?);
    let unix_socket = omic::socket::bind()?;

    let _io = main_loop.add_io(unix_socket, IoFlags::IN, move |unix_socket| {
        let closure = |stream: &mut UnixSeqpacketConn| -> Result<(), anyhow::Error> {
            let mut bytes = [0; MAX_MESSAGE_SIZE];
            stream.recv(&mut bytes)?;
            let message = Request::from_bytes(&bytes)?;

            // TODO: store current state
            match message {
                Request::Connect { address, port } => {
                    tracing::info!("attempting to connect to {}:{}", address, port);
                    let addr = format!("{}:{}", address, port);

                    udp_socket.connect(addr)?;
                    udp_socket.send(&[UdpSocketMessage::Connect as u8])?;
                    tracing::info!("connection established, connect byte sent");
                }
                Request::Disconnect => {
                    tracing::info!("sending disconnect signal");
                    udp_socket.send(&[UdpSocketMessage::Disconnect as u8])?;
                }
                // TODO: add current state query
                Request::Query => todo!(),
                Request::Noop => {}
            }

            Ok(())
        };

        if let Ok((mut stream, _)) = unix_socket.accept_unix_addr() {
            let _ = match closure(&mut stream) {
                Ok(_) => stream.send(&bincode::serialize(&Response::Ok).unwrap()),
                Err(e) => {
                    let error = format!("error processing message: {}", e);
                    tracing::error!(error);
                    stream.send(&bincode::serialize(&Response::Error(error)).unwrap())
                }
            };
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
