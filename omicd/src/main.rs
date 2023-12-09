use anyhow::Context;
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::{UdpSocketMessage, MAX_MESSAGE_SIZE},
    message::{Request, Response, State},
    pipewire::{get_pw_params, PipewireContext},
};
use pipewire::{MainLoop, Signal, WeakMainLoop};
use std::{
    io::Write,
    mem::ManuallyDrop,
    net::{TcpStream, ToSocketAddrs, UdpSocket},
    rc::Weak,
    sync::{Arc, Mutex},
};
use tracing::info;
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
        Request::Connect { address, port } => {
            let mut res = Response::Ok;
            info!("Attempting to connect to {}:{}.", address, port);
            let addr = format!("{}:{}", address, port);
            state.connection =
                match TcpStream::connect_timeout(&addr.parse()?, std::time::Duration::from_secs(5))
                {
                    Ok(mut connection) => {
                        info!("Connected succesfully.");
                        connection
                            .write_all(&[UdpSocketMessage::Connect as u8])
                            .unwrap();
                        connection.write_all(&state.port.to_be_bytes()).unwrap();
                        Some(connection)
                    }
                    Err(_) => {
                        info!("Timed out connecting to {}.", addr);
                        res = Response::Error("Failed to connect.".to_owned());
                        None
                    }
                };
            Ok(res)
        }

        Request::Disconnect => {
            match state.connection.as_ref() {
                Some(mut connection) => {
                    info!("Disconnected from server.");
                    connection
                        .write_all(&[UdpSocketMessage::Disconnect as u8])
                        .unwrap();
                    connection.shutdown(std::net::Shutdown::Both).unwrap();
                    state.connection = None;
                    Ok(Response::Ok)
                }
                None => {
                    info!("Received a disconnect while not connected.");
                    Ok(Response::Error("Wasn't connected.".to_owned()))
                }
            } // ret
        }

        Request::Status => {
            // let connection = state.connection.as_ref();
            // let addr = connection.unwrap().
            Ok(Response::Ok)
        }
        Request::Noop => todo!(),
    }
}

fn exit(main_loop_weak: &WeakMainLoop) {
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
            tracing::error!("couldn't connect to journald: {}", e);
        }
    }
    let unix_socket = omic::socket::bind()?;
    let main_loop = MainLoop::new()?;
    // this has to be done before passing to context
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGINT, move || exit(&main_loop_weak));
    let main_loop_weak = main_loop.downgrade();
    let _sig = main_loop.add_signal_local(Signal::SIGTERM, move || exit(&main_loop_weak));
    let context = pipewire::Context::new(&main_loop)?;
    let core = context.connect(None)?;
    let udp_socket = UdpSocket::bind("0.0.0.0:0").context("couldn't bind to address")?;
    udp_socket.set_nonblocking(true)?;
    let udp_port = udp_socket.local_addr().unwrap().port();
    info!("UDP open on port {}", udp_port);
    let state = State {
        socket: unix_socket,
        connection: None,
        port: udp_port,
    };

    let stream = omic::pipewire::create_stream(&core)?;
    let _s = ManuallyDrop::new(omic::pipewire::register_callbacks(&stream, udp_socket)?);

    let _io = main_loop.add_io(state, IoFlags::IN, move |state| {
        if let Ok((mut stream, _)) = state.socket.accept_unix_addr() {
            let _ = match read_request(state, &mut stream) {
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
