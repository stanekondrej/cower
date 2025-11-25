use anyhow::anyhow;
use native_tls::Identity;
use std::{
    env, fs,
    io::Read,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread::{self, JoinHandle},
};

use clap::Parser;

use cower_common::Acceptor;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:9989";

#[derive(Parser)]
#[command(about, long_about)]
struct Args {
    /// Socket address to bind to
    #[arg(short, long, default_value_t = String::from(DEFAULT_BIND_ADDR))]
    addr: String,

    /// Path to identity file
    #[arg(long)]
    ident_path: Option<PathBuf>,

    /// Password to identity file
    #[arg(long)]
    ident_pass: Option<String>,
}

fn spawn_handler_thread(acceptor: Acceptor, stream: TcpStream) -> JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || {
        let mut stream = acceptor.accept(stream)?;
        let msg = stream.receive()?;

        dbg!(msg);
        todo!("Implement message handling functionality");
    })
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    _ = args;

    let ident_path = args
        .ident_path
        .or_else(|| env::var("COWER_IDENT_PATH").ok().map(PathBuf::from))
        .ok_or(anyhow!("Missing path to identity file"))?;

    let ident_pass = args
        .ident_pass
        .or_else(|| env::var("COWER_IDENT_PASS").ok())
        .ok_or(anyhow!("Missing password to identity file"))?;

    let mut ident_buf = vec![];
    let mut identity = fs::File::open(ident_path)?;
    identity.read_to_end(&mut ident_buf)?;

    let identity = Identity::from_pkcs12(&ident_buf, &ident_pass)?;

    let acceptor = Acceptor::new(identity)?;
    let listener = TcpListener::bind(args.addr)?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                _ = spawn_handler_thread(acceptor, stream);
            }
            Err(why) => println!("Failed to accept connection: {why}"),
        }
    }

    Ok(())
}
