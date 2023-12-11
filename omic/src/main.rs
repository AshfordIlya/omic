use clap::{arg, Parser, Subcommand};
use omic::message::{Request, Response};
use tracing::{error, info};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Connect to an omic server.
    Connect {
        /// IP Address of an omic server.
        #[arg(long, short)]
        address: String,
        /// Port to connect on.
        #[arg(long, short)]
        port: String,
    },
    /// Disconnect from current omic server.
    Disconnect,
    /// Query the status of the omic server.
    Status,
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .compact()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .init();

    let args = Args::parse();

    let request = match args.command {
        Command::Connect { address, port } => Request::Connect { address, port },
        Command::Disconnect => Request::Disconnect,
        Command::Status => Request::Status,
    };

    let response = omic::socket::Socket::create_request()
        .request(request)
        .send_with_response();

    match response {
        Ok(Response::Ok) => {}
        Ok(Response::Error(err)) => error!("{}", err),
        Ok(Response::Connection { addr }) => info!("Connected to {addr}"),
        Err(_) => error!("Socket error, check if daemon is running."),
    }
    // write to unix socket
    Ok(())
}
