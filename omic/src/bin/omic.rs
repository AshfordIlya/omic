use clap::{arg, Parser, Subcommand};
use omic::message::{Request, Response};
use std::{
    io::{Read, Write},
    net::Shutdown,
};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
enum Command {
    Connect {
        #[arg(long)]
        address: String,

        #[arg(long)]
        port: String,
    },
    Disconnect,
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();
    let args = Args::parse();
    let mut unix_socket = omic::socket::connect()?;

    match args.command {
        Command::Connect { address, port } => {
            let buffer = bincode::serialize(&Request::Connect { address, port })?;
            unix_socket.write_all(&buffer)?;
        }
        Command::Disconnect => {
            let buffer = bincode::serialize(&Request::Disconnect)?;
            unix_socket.write_all(&buffer)?;
        }
    }

    // shutdown the write end as we're done, and then wait to read.
    unix_socket.shutdown(Shutdown::Write)?;
    let mut read_buffer = Vec::new();
    unix_socket.read_to_end(&mut read_buffer)?;
    let response: Response = bincode::deserialize(&read_buffer)?;
    match response {
        Response::Ok => {}
        Response::Error(err) => tracing::error!(err),
    }

    // write to unix socket
    Ok(())
}
