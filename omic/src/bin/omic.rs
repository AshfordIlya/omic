use clap::{arg, Parser, Subcommand};
use omic::message::Message;
use std::{io::Write, time::Duration};

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
    unix_socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    match args.command {
        Command::Connect { address, port } => {
            let buffer = bincode::serialize(&Message::Connect { address, port })?;
            unix_socket.write_all(&buffer)?;
        }
        Command::Disconnect => {
            let buffer = bincode::serialize(&Message::Disconnect)?;
            unix_socket.write_all(&buffer)?;
        }
    }

    // write to unix socket
    Ok(())
}
