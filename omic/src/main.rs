use clap::{arg, Parser, Subcommand};
use omic::message::{Request, Response};
use std::{
    io::Write,
    net::{Ipv4Addr, TcpStream},
    process::exit,
    time::Duration,
};
use tracing::{error, info};
#[derive(Parser)]
struct Args {
    /// IP of womic server.
    ip: Ipv4Addr,
}

pub enum RequestId {
    SetPlatform = 101,
    SetCodec = 102,
    Start = 103,
    Poll = 105,
}
// womic tcp bytes
const LEN_SET_PLATFORM: i32 = 6;
const LEN_SET_CODEC: i32 = 6;
const LEN_START: i32 = 0;
const LEN_POLL: i32 = 0;
const CODEC_OPUS: u8 = 2;
const RATE_4800: u8 = 2;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .compact()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .init();

    let args = Args::parse();

    let response = omic::socket::Socket::create_request()
        .request(Request::Status)
        .send_with_response();
    if let Ok(Response::Connection { port }) = response {
        let mut tcp = TcpStream::connect_timeout(
            &std::net::SocketAddr::new(std::net::IpAddr::V4(args.ip), 8125),
            Duration::from_secs(2),
        )?;

        // set platform
        tcp.write_all(&[RequestId::SetPlatform as u8])?;
        tcp.write_all(&LEN_SET_PLATFORM.to_be_bytes())?;
        let _sz: u8 = 4;
        tcp.write_all(&_sz.to_be_bytes())?;
        tcp.write_all(&_sz.to_be_bytes())?;
        let _sz: i32 = 100663296;
        tcp.write_all(&_sz.to_be_bytes())?;
        // set codec
        tcp.write_all(&[RequestId::SetCodec as u8])?;
        tcp.write_all(&LEN_SET_CODEC.to_be_bytes())?;
        tcp.write_all(&CODEC_OPUS.to_be_bytes())?;
        tcp.write_all(&RATE_4800.to_be_bytes())?;
        tcp.write_all(&port.to_be_bytes())?;
        info!("PORT {}", port);
        // start

        tcp.write_all(&[RequestId::Start as u8])?;
        tcp.write_all(&LEN_START.to_be_bytes())?;

        loop {
            tcp.write_all(&[RequestId::Poll as u8])?;
            tcp.write_all(&LEN_POLL.to_be_bytes())?;
            std::thread::sleep(Duration::from_secs(3));
        }
    } else {
        error!("Error getting UDP port from daemon");
        exit(-1);
    }
    Ok(())
}
