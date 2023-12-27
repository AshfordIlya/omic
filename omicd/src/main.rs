use anyhow::{Context, Error};
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::{Message, MAX_MESSAGE_SIZE},
    message::{Request, Response, State},
    pipewire::{get_pw_params, CallbackData},
};
use opus::Decoder;
use pipewire::{stream::Stream, MainLoop, Signal, WeakMainLoop};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    thread,
    time::Duration,
};
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use uds::UnixSeqpacketConn;

fn read_request(
    state: &mut State,
    stream: &mut UnixSeqpacketConn,
) -> Result<Response, anyhow::Error> {
    let mut bytes = [0; MAX_MESSAGE_SIZE];
    stream.recv(&mut bytes)?;
    let message = Request::from_bytes(&bytes)?;
    match message {
        Request::Status => {
            info!("Sent UDP information over socket");
            Ok(Response::Connection {
                port: state.port as i32,
            })
        }
    }
}

fn exit(main_loop_weak: &WeakMainLoop) {
    match omic::socket::disconnect() {
        Ok(_) => {}
        Err(e) => {
            error!("error disconnecting from socket: {}", e)
        }
    };
    info!("disconnected");
    if let Some(main_loop) = main_loop_weak.upgrade() {
        main_loop.quit();
    }
    info!("main loop finished");
}

fn main() -> Result<(), anyhow::Error> {
    pipewire::init();
    let registry = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .compact()
            .without_time()
            .with_file(false)
            .with_line_number(false)
            .with_target(false),
    );
    match tracing_journald::layer() {
        Ok(subscriber) => {
            registry.with(subscriber).init();
        }
        Err(e) => {
            registry.init();
            error!("couldn't connect to journald: {}", e);
        }
    }
    let unix_socket = omic::socket::bind()?;
    let main_loop = MainLoop::new()?;
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGINT, move || exit(&main_loop_weak));
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGTERM, move || exit(&main_loop_weak));
    let context = pipewire::Context::new(&main_loop)?;
    let core = context.connect(None)?;
    let udp_socket = UdpSocket::bind("0.0.0.0:0").context("couldn't bind to address")?;
    udp_socket.set_nonblocking(true)?;
    let udp_port = udp_socket.local_addr()?.port();
    info!("UDP open on port {}", udp_port);
    let state = State {
        socket: unix_socket,
        port: udp_port,
    };

    let stream = omic::pipewire::create_stream(&core)?;
    stream.connect(
        Direction::Output,
        None,
        pipewire::stream::StreamFlags::MAP_BUFFERS
            | pipewire::stream::StreamFlags::AUTOCONNECT
            | pipewire::stream::StreamFlags::RT_PROCESS,
        &mut get_pw_params(),
    )?;

    let data = CallbackData {
        udp: udp_socket,
        decoder: Decoder::new(48000, opus::Channels::Mono).unwrap(),
    };

    let _s = omic::pipewire::register_callbacks(&stream, data)?;
    let _io = main_loop.add_io(state, IoFlags::IN, move |state| {
        if let Ok((mut socket, _)) = state.socket.accept_unix_addr() {
            let _ = match read_request(state, &mut socket) {
                Ok(r) => socket.send(&r.to_bytes().unwrap()),
                Err(e) => {
                    error!("{}", e);
                    socket.send(&Response::Error(e.to_string()).to_bytes().unwrap())
                }
            };
        }
    });

    main_loop.run();
    Ok(())
}
