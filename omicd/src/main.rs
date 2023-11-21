use anyhow::Context;
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::{UdpSocketMessage, MAX_MESSAGE_SIZE},
    message::{Connection, Request, Response, State},
    pipewire::{get_pw_params, PipewireContext},
};
use pipewire::{MainLoop, Signal, WeakMainLoop};
use std::{
    mem::ManuallyDrop,
    net::UdpSocket,
    sync::{Arc, Mutex},
};
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

    let state = State {
        socket: unix_socket,
        connection: Mutex::new(Connection {
            status: omic::message::Status::Disconnected,
            address: None,
            port: None,
        }),
    };

    let _io = main_loop.add_io(state, IoFlags::IN, move |state| {
        let closure = |stream: &mut UnixSeqpacketConn| -> Result<Response, anyhow::Error> {
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

                    let mut connection = state.connection.lock().unwrap();
                    connection.status = omic::message::Status::Connected;
                    connection.address = Some(address);
                    connection.port = Some(port);
                    Ok(Response::Ok)
                }
                Request::Disconnect => {
                    tracing::info!("sending disconnect signal");
                    udp_socket.send(&[UdpSocketMessage::Disconnect as u8])?;
                    let mut connection = state.connection.lock().unwrap();
                    connection.status = omic::message::Status::Disconnected;
                    connection.address = None;
                    connection.port = None;
                    Ok(Response::Ok)
                }
                // TODO: add current state query
                Request::Status => {
                    let connection = state.connection.lock().unwrap();
                    Ok(Response::Connection(connection.clone()))
                }
                Request::Noop => todo!(),
            }
        };

        if let Ok((mut stream, _)) = state.socket.accept_unix_addr() {
            let _ = match closure(&mut stream) {
                Ok(r) => stream.send(&r.to_bytes().unwrap()),
                Err(e) => {
                    let error = format!("error processing message: {}", e);
                    tracing::error!(error);
                    stream.send(&Response::Error(error).to_bytes().unwrap())
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
