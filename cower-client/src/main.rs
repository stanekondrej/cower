use std::{env, fs, io::Read};

use clap::Parser;

use cower_common::prelude::*;
use native_tls::Certificate;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Force direct connection to target. Fails if provided address belongs to a server
    #[arg(short, long, default_value_t = false)]
    direct: bool,

    /// Path to a custom certificate
    #[arg(short, long)]
    cert_path: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let cert_path = args.cert_path.or_else(|| env::var("COWER_CERT").ok());
    let cert: Option<Certificate> = if let Some(cert_path) = cert_path {
        let mut file = fs::File::open(&cert_path)?;
        let mut buf = vec![];
        _ = file.read_to_end(&mut buf)?;

        Some(Certificate::from_pem(&buf)?)
    } else {
        eprintln!("Running without custom certificate. This might cause you trouble!");

        None
    };

    let mut conn = Connection::connect("127.0.0.1:9989", "localhost", cert)?;

    let msg = Message::StartMessage {
        resource_name: "my_resource".to_owned(),
    };
    conn.send(&msg)?;

    Ok(())
}
