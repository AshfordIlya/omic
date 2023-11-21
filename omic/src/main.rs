use clap::{arg, Parser, Subcommand};
use omic::message::{Request, Response};

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
    tracing_subscriber::fmt().init();
    let args = Args::parse();

    let request = match args.command {
        Command::Connect { address, port } => Request::Connect { address, port },
        Command::Disconnect => Request::Disconnect,
        Command::Status => Request::Status,
    };

    let response = omic::socket::Socket::create_request()
        .request(request)
        .send_with_response()?;

    match response {
        Response::Ok => {}
        Response::Error(err) => tracing::error!(err),
        Response::Connection(connection) => match connection.status {
            omic::message::Status::Connected => {
                tracing::info!(
                    "Connected to {0}:{1}",
                    connection.address.unwrap(),
                    connection.port.unwrap()
                );
            }
            omic::message::Status::Disconnected => {
                tracing::info!("Not currently connected to any server.");
            }
            omic::message::Status::Error => todo!(),
        },
    }

    // write to unix socket
    Ok(())
}
