use std::{
    env, fs,
    io::Read,
    net::TcpListener,
    thread::{self, JoinHandle},
};

use cower_common::prelude::*;
use native_tls::Identity;

fn main() -> anyhow::Result<()> {
    let identity_pass = env::var("COWER_IDENT_PASS")?;
    let mut buf: Vec<u8> = vec![];
    {
        let identity_env = env::var("COWER_IDENT")?;
        let mut identity_file = fs::File::open(identity_env)?;

        identity_file.read_to_end(&mut buf)?;
    }

    let identity = Identity::from_pkcs12(&buf, &identity_pass)?;
    let acceptor = cower_common::Acceptor::new(identity)?;

    let listener = TcpListener::bind("0.0.0.0:9989")?;
    eprintln!("Bound to port 9989");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let _: JoinHandle<anyhow::Result<()>> = thread::spawn(move || {
                    let mut stream = acceptor.accept(stream)?;
                    let message = stream.receive()?;

                    match message {
                        Message::StartMessage { resource_name } => {
                            println!("Received start message");
                            dbg!(&resource_name);
                        }
                    }

                    Ok(())
                });
            }
            Err(why) => {
                eprintln!("Failed to accept stream: {why}")
            }
        }
    }

    Ok(())
}
