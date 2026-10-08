use std::future::Future;
use std::pin::Pin;

use futures_util::stream::BoxStream;

use super::certificate_trust::CertificatePin;
use super::credential_store::PairingToken;
use super::remote_dispatcher::RequestId;
use super::tv_address::{TargetError, TvHost};
use crate::domain::RemoteAction;

pub type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ProbeObservation {
    pub pin: CertificatePin,
    pub name: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvSessionError {
    Target(TargetError),
    Offline,
    Timeout,
    Tls,
    CertificateChanged,
    RemoteChannel,
    PairingDenied,
    TokenRejected,
    PairingTokenMissing,
    Protocol,
    UncertainWrite,
    UnsupportedAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TvSessionEvent {
    Written(RequestId),
    NotSent(RequestId),
    Uncertain(RequestId),
    Disconnected,
    TokenRejected,
}

pub trait TvSessionControl: Send {
    fn try_click(&self, id: RequestId, action: RemoteAction) -> bool;
    fn abort(&mut self);
}

pub struct ActiveTvSession {
    pub control: Box<dyn TvSessionControl>,
    pub events: BoxStream<'static, TvSessionEvent>,
}

pub trait TvConnection: Send {
    fn await_authorized(
        &mut self,
        reconnecting: bool,
    ) -> SessionFuture<'_, Result<Option<PairingToken>, TvSessionError>>;
    fn start(self: Box<Self>) -> ActiveTvSession;
}

pub trait TvGateway: Send + Sync {
    fn probe(
        &self,
        host: TvHost,
    ) -> SessionFuture<'static, Result<ProbeObservation, TvSessionError>>;
    fn connect(
        &self,
        host: TvHost,
        pin: CertificatePin,
        token: Option<PairingToken>,
    ) -> SessionFuture<'static, Result<Box<dyn TvConnection>, TvSessionError>>;
}
