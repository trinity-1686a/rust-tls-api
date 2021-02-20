//! [`tls_api`] implementation for [`rustls`].

#![deny(broken_intra_doc_links)]

mod acceptor;
mod connector;
mod error;
mod handshake;
mod stream;

use tls_api::ImplInfo;

pub use acceptor::TlsAcceptor;
pub use acceptor::TlsAcceptorBuilder;
pub use connector::TlsConnector;
pub use connector::TlsConnectorBuilder;

pub(crate) use error::Error;
pub(crate) use stream::TlsStream;

pub(crate) fn info() -> ImplInfo {
    ImplInfo {
        name: "rustls",
        version: "unknown",
    }
}
