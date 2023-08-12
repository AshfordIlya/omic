use clap::{arg, Parser, Subcommand};
use omic::message::{Request, Response};

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

    let request = match args.command {
        Command::Connect { address, port } => Request::Connect { address, port },
        Command::Disconnect => Request::Disconnect,
    };

    let response = omic::socket::Socket::create_request()
        .request(request)
        .send_with_response()?;

    match response {
        Response::Ok => {}
        Response::Error(err) => tracing::error!(err),
    }

    // write to unix socket
    Ok(())
}
