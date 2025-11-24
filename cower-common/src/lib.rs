//! Common code shared by all the other `cower` modules. This library consists mainly of netcode
//! and serde mechanisms.

#![deny(missing_docs)]

pub mod message;
pub mod prelude;

use message::Message;

use core::str;
use native_tls::{Certificate, Identity, TlsAcceptor, TlsStream};
use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    net::{self, TcpStream},
    result,
};

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

impl Connection<()> {
    /// Send a message over the connection
    pub fn send(&mut self, message: &dyn Message) -> crate::Result<()> {
        let buf = message.serialize_data()?;

        self.stream.write_all(&buf)?;
        Ok(())
    }

    /// Receive a message over the connection
    pub fn receive(&mut self) -> crate::Result<Box<dyn Message>> {
        // FIXME: this is broken right now

        let mut buf = [0; message::MAX_MESSAGE_LENGTH];
        self.stream.read_exact(&mut buf)?;

        todo!()
    }

    /// Connects to the given server
    pub fn connect(
        addr: net::SocketAddr,
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
pub struct Acceptor(TlsAcceptor);

impl Acceptor {
    /// Constructs a new acceptor with sane TLS configuration.
    pub fn new(identity: Identity) -> crate::Result<Acceptor> {
        let acceptor = TlsAcceptor::builder(identity)
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
