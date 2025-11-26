//! Common code shared by all the other `cower` modules. This library consists mainly of netcode
//! and serde mechanisms.

#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]

pub mod message;
pub mod prelude;

use message::Message;

use core::str;
use native_tls::{Certificate, Identity, TlsAcceptor, TlsStream};
use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    net::{TcpStream, ToSocketAddrs},
    result,
};

use crate::message::{HEADER_SIZE, MessageHeader};

/// Error type returned by all the different functions this library provides
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    IOFailure(#[from] io::Error),
    #[error("TLS error")]
    TLSFailure(#[from] native_tls::Error),
    #[error("TLS handshake error")]
    TLSHandshakeFailure(#[from] native_tls::HandshakeError<TcpStream>),
    #[error("message too long")]
    MesssageTooBig,
    #[error("unknown message type")]
    UnknownMessage,
    #[error("invalid UTF-8")]
    InvalidUtf8(#[from] str::Utf8Error),
}

/// The result type returned by this library's functions
pub type Result<T> = result::Result<T, crate::Error>;

/// The end of a connection that acts as the initiator
pub struct Client;
/// The end of a connection that acts as the acceptor
pub struct Server;

/// An encrypted connection between `cower` programs
///
/// # Initialization
///
/// If you need to connect to a client, use [`Connection::connect`]. If you need to accept a
/// connection from a client, use [`Acceptor`] instead.
pub struct Connection<T> {
    stream: TlsStream<TcpStream>,
    _0: PhantomData<T>,
}

impl<T> Connection<T> {
    /// Send a message over the connection
    pub fn send(&mut self, message: &Message) -> crate::Result<()> {
        let header_buf = message.create_header()?.serialize();
        let message_buf = message.serialize_payload()?;

        let buf = [&header_buf, message_buf.iter().as_slice()].concat();

        self.stream.write_all(&buf)?;
        Ok(())
    }

    /// Receive a message over the connection
    pub fn receive(&mut self) -> crate::Result<Message> {
        let mut buf = [0; HEADER_SIZE as usize];
        self.stream.read_exact(&mut buf)?;

        let header = MessageHeader::deserialize(&buf[0..HEADER_SIZE as usize])?;

        let mut data_buf = vec![0; header.length.into()];
        self.stream.read_exact(&mut data_buf)?;

        Message::deserialize(&header, &data_buf)
    }
}

impl Connection<()> {
    /// Connects to the given server
    pub fn connect<A: ToSocketAddrs>(
        addr: A,
        domain: &str,
        custom_cert: Option<Certificate>,
    ) -> Result<Connection<Client>> {
        let stream = TcpStream::connect(addr)?;
        let mut connector = native_tls::TlsConnector::builder();
        if let Some(cert) = custom_cert {
            connector.add_root_certificate(cert);
        }
        let connector = connector.build()?;

        let tls_stream = connector.connect(domain, stream)?;

        Ok(Connection {
            stream: tls_stream,
            _0: PhantomData,
        })
    }
}

/// Accepts and initiates connections, verifies the identity of clients
#[derive(Clone)]
pub struct Acceptor(TlsAcceptor);

impl Acceptor {
    /// Constructs a new acceptor with sane TLS configuration.
    pub fn new(identity: Identity) -> crate::Result<Acceptor> {
        let acceptor = TlsAcceptor::builder(identity)
            // remove this if this causes problems for older platforms
            .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
            .build()?;

        Ok(Self(acceptor))
    }

    /// Accepts an incoming connection. Pass the stream in before writing anything to it.
    pub fn accept(&self, stream: TcpStream) -> crate::Result<Connection<Server>> {
        let tls_stream = self.0.accept(stream)?;

        Ok(Connection {
            stream: tls_stream,
            _0: PhantomData,
        })
    }
}

#[cfg(test)]
mod acceptor_tests {
    use std::{
        net::{TcpListener, ToSocketAddrs},
        thread::{self, JoinHandle},
    };

    use native_tls::{Certificate, Identity};

    use crate::message::Message;

    use super::{Acceptor, Connection};

    const IDENT_FILE: &[u8] = include_bytes!("../../test-keys/identity.p12");
    const IDENT_PASS: &str = include_str!("../../test-keys/creds.asc");
    const CUSTOM_CERT: &[u8] = include_bytes!("../../test-keys/cert.crt");

    fn setup_test() -> crate::Result<(Acceptor, Certificate)> {
        let cert = Certificate::from_pem(CUSTOM_CERT)?;
        let identity = Identity::from_pkcs12(IDENT_FILE, IDENT_PASS.trim())?;

        Ok((Acceptor::new(identity)?, cert))
    }

    fn get_local_addr() -> Option<impl ToSocketAddrs> {
        let port = port_check::free_local_ipv4_port()?;

        Some(("127.0.0.1", port))
    }

    #[test]
    fn accept_connection() -> crate::Result<()> {
        let (acceptor, cert) = setup_test()?;

        let addr = get_local_addr().expect("failed to get local address");
        let listener = TcpListener::bind(&addr)?;
        let handle: JoinHandle<crate::Result<()>> = thread::spawn(move || {
            _ = Connection::connect(&addr, "localhost", Some(cert))?;

            Ok(())
        });

        let stream = listener
            .incoming()
            .next()
            .expect("no next stream (this should never happen)")
            .expect("failed to accept stream");
        _ = acceptor.accept(stream)?;

        _ = handle.join().expect("associated thread panicked");

        Ok(())
    }

    #[test]
    fn accept_message() -> crate::Result<()> {
        let (acceptor, cert) = setup_test()?;

        const RESOURCE_NAME: &str = "my_resource";

        let addr = get_local_addr().expect("failed to get local address");
        let listener = TcpListener::bind(&addr)?;
        let handle: JoinHandle<crate::Result<()>> = thread::spawn(move || {
            let mut conn = Connection::connect(&addr, "localhost", Some(cert))?;

            let msg = Message::StartMessage {
                resource_name: RESOURCE_NAME.to_owned(),
            };
            conn.send(&msg)?;

            Ok(())
        });

        let stream = listener
            .incoming()
            .next()
            .expect("no next stream (this should never be reached)")
            .expect("failed to accept stream");
        let mut conn = acceptor.accept(stream)?;
        let msg = conn.receive()?;

        #[allow(irrefutable_let_patterns)] // TODO: remove this when more message types are added
        if let Message::StartMessage { resource_name } = msg {
            assert_eq!(&resource_name, RESOURCE_NAME);
        } else {
            panic!("received different message type")
        }

        _ = handle.join().expect("associated thread panicked");

        Ok(())
    }
}
