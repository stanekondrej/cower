use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
};

use clap::Parser;

use cower_common::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Force direct connection to target. Fails if provided address belongs to a server
    #[arg(short, long, default_value_t = false)]
    direct: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    let conn = Connection::connect(addr, "", None)?;

    Ok(())
}
