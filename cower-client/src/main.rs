use std::error::Error;

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
    let mut conn = Connection::connect("127.0.0.1:9989", "localhost", None)?;

    let msg = Message::StartMessage {
        resource_name: "my_resource".to_owned(),
    };
    conn.send(&msg)?;

    Ok(())
}
