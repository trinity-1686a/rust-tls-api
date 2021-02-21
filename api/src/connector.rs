use std::marker;

use crate::connector_box::TlsConnectorBox;
use crate::connector_box::TlsConnectorTypeImpl;
use crate::socket::AsyncSocket;
use crate::stream::TlsStream;
use crate::BoxFuture;
use crate::ImplInfo;
use crate::TlsConnectorType;
use crate::TlsStreamDyn;
use crate::TlsStreamWithSocket;

/// A builder for `TlsConnector`s.
pub trait TlsConnectorBuilder: Sized + Sync + Send + 'static {
    /// Result of connector to be build.
    type Connector: TlsConnector;

    /// Type of the underlying builder.
    type Underlying;

    /// Get the underlying builder.
    ///
    /// API intentionally exposes the underlying acceptor builder to allow fine tuning
    /// not possible in common API.
    fn underlying_mut(&mut self) -> &mut Self::Underlying;

    /// Set ALPN-protocols to negotiate.
    ///
    /// This operations fails is not [`TlsConnector::SUPPORTS_ALPN`].
    fn set_alpn_protocols(&mut self, protocols: &[&[u8]]) -> crate::Result<()>;

    /// Should hostname verification be performed?
    /// Use carefully, it opens the door to MITM attacks.
    fn set_verify_hostname(&mut self, verify: bool) -> crate::Result<()>;

    /// Add trusted root certificate. By default connector supports only
    /// global trusted root.
    ///
    /// Param is DER-encoded X.509 certificate.
    fn add_root_certificate(&mut self, cert: &[u8]) -> crate::Result<()>;

    /// Finish the acceptor constructon.
    fn build(self) -> crate::Result<Self::Connector>;
}

/// A builder for client-side TLS connections.
pub trait TlsConnector: Sized + Sync + Send + 'static {
    /// Type of the builder for this connector.
    type Builder: TlsConnectorBuilder<Connector = Self>;

    /// Type of the underlying connector.
    type Underlying;

    /// `crate::TlsStream<tls_api::AsyncSocketBox>`.
    ///
    /// In the world of HKT this would be:
    ///
    /// ```ignore
    /// type TlsStream<S: TlsStreamDyn> : TlsStreamWithSocketDyn<S>;
    /// ```
    type TlsStream: TlsStreamDyn;

    /// Get the underlying builder.
    ///
    /// API intentionally exposes the underlying acceptor builder to allow fine tuning
    /// not possible in common API.
    fn underlying_mut(&mut self) -> &mut Self::Underlying;

    /// Is it implemented? When `false` all operations return an error.
    ///
    /// At the moment of writing, there are two crates which return `false` here:
    /// * `tls-api-stub`, dummy implementation is not meant to be instantiated
    /// * `tls-api-security-framework`, `true` only on macOS and iOS, `false` elsewhere
    const IMPLEMENTED: bool;

    /// Whether this implementation supports ALPN negotiation.
    const SUPPORTS_ALPN: bool;

    /// Implementation info.
    fn info() -> ImplInfo;

    /// New builder for the acceptor.
    fn builder() -> crate::Result<Self::Builder>;

    /// Dynamic (without type parameter) version of the connector.
    ///
    /// This function returns a connector type, which can be used to constructor connectors.
    const TYPE_DYN: &'static dyn TlsConnectorType =
        &TlsConnectorTypeImpl::<Self>(marker::PhantomData);

    /// Dynamic (without type parameter) version of the connector.
    fn into_dyn(self) -> TlsConnectorBox {
        TlsConnectorBox::new(self)
    }

    /// Connect.
    ///
    /// Returned future is resolved when the TLS-negotiation completes,
    /// and the stream is ready to send and receive.
    fn connect_with_socket<'a, S>(
        &'a self,
        domain: &'a str,
        stream: S,
    ) -> BoxFuture<'a, crate::Result<TlsStreamWithSocket<S>>>
    where
        S: AsyncSocket;

    /// Connect.
    ///
    /// Returned future is resolved when the TLS-negotiation completes,
    /// and the stream is ready to send and receive.
    fn connect<'a, S>(
        &'a self,
        domain: &'a str,
        stream: S,
    ) -> BoxFuture<'a, crate::Result<TlsStream>>
    where
        S: AsyncSocket,
    {
        BoxFuture::new(async move {
            self.connect_with_socket(domain, stream)
                .await
                .map(TlsStream::new)
        })
    }

    /// Connect.
    ///
    /// Returned future is resolved when the TLS-negotiation completes,
    /// and the stream is ready to send and receive.
    fn connect_impl_tls_stream<'a, S>(
        &'a self,
        domain: &'a str,
        stream: S,
    ) -> BoxFuture<'a, crate::Result<Self::TlsStream>>
    where
        S: AsyncSocket;
}

/// Common part of all connectors. Poor man replacement for HKT.
#[macro_export]
macro_rules! spi_connector_common {
    () => {
        fn connect_with_socket<'a, S>(
            &'a self,
            domain: &'a str,
            stream: S,
        ) -> $crate::BoxFuture<'a, $crate::Result<$crate::TlsStreamWithSocket<S>>>
        where
            S: $crate::AsyncSocket,
        {
            $crate::BoxFuture::new(async move {
                let crate_tls_stream: crate::TlsStream<S> =
                    self.connect_impl(domain, stream).await?;
                Ok($crate::TlsStreamWithSocket::new(crate_tls_stream))
            })
        }

        fn connect_impl_tls_stream<'a, S>(
            &'a self,
            domain: &'a str,
            stream: S,
        ) -> tls_api::BoxFuture<'a, tls_api::Result<Self::TlsStream>>
        where
            S: AsyncSocket,
        {
            tls_api::BoxFuture::new(self.connect_impl(domain, tls_api::AsyncSocketBox::new(stream)))
        }
    };
}
