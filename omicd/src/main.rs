use anyhow::{Context, Error};
use libspa::{flags::IoFlags, Direction};
use omic::{
    constants::{Message, MAX_MESSAGE_SIZE},
    message::{Request, Response, State},
    pipewire::get_pw_params,
};
use pipewire::{MainLoop, Signal, WeakMainLoop};
use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
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
        Request::Connect { address, port } => {
            info!("Attempting to connect to {}:{}.", address, port);
            let addr = format!("{}:{}", address, port);
            match TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(5)) {
                Ok(mut connection) => {
                    info!("Connected succesfully.");
                    connection.write_all(&[Message::Connect as u8])?;
                    connection.write_all(&state.port.to_be_bytes())?;
                    connection.set_read_timeout(Some(Duration::from_millis(150)))?;
                    state.connection = Some(connection);
                    Ok(Response::Ok)
                }
                Err(_) => {
                    state.connection = None;
                    Err(Error::msg(format!("Timed out connecting to {}", addr)))
                }
            }
        } // ret

        Request::Disconnect => match state.connection.as_ref() {
            Some(mut connection) => {
                info!("Disconnected from server.");
                connection.write_all(&[Message::Disconnect as u8])?;
                connection.shutdown(std::net::Shutdown::Both)?;
                state.connection = None;
                Ok(Response::Ok)
            }
            None => {
                info!("Received a disconnect while not connected.");
                Err(Error::msg("Wasn't connected."))
            }
        }, // ret

        Request::Status => match state.connection.as_ref() {
            Some(mut connection) => match connection.write(&[Message::Hello as u8]) {
                Ok(_) => {
                    let mut buf = [0; 1];
                    connection
                        .read_exact(&mut buf)
                        .map_err(|_| Error::msg("Not connected."))?;
                    Ok(Response::Connection {
                        addr: connection.peer_addr()?.to_string(),
                    })
                }
                Err(_) => Err(Error::msg("Not connected.")),
            },
            None => Err(Error::msg("Not connected.")),
        }, // ret
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
    // this has to be done before passing to context
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
        connection: None,
        port: udp_port,
    };

    let stream = omic::pipewire::create_stream(&core)?;
    let _s = omic::pipewire::register_callbacks(&stream, udp_socket)?;

    let _io = main_loop.add_io(state, IoFlags::IN, move |state| {
        if let Ok((mut stream, _)) = state.socket.accept_unix_addr() {
            let _ = match read_request(state, &mut stream) {
                Ok(r) => stream.send(&r.to_bytes().unwrap()),
                Err(e) => {
                    error!("{}", e);
                    stream.send(&Response::Error(e.to_string()).to_bytes().unwrap())
                }
            };
        }
    });

    stream.connect(
        Direction::Output,
        None,
        pipewire::stream::StreamFlags::MAP_BUFFERS
            | pipewire::stream::StreamFlags::RT_PROCESS
            | pipewire::stream::StreamFlags::AUTOCONNECT,
        &mut get_pw_params(),
    )?;
    main_loop.run();
    Ok(())
}
