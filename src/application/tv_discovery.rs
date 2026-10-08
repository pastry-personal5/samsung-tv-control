use super::tv_address::TvHost;
use super::tv_session::SessionFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    Permission,
    Unavailable,
}

/// Discovery results are untrusted hints and never establish TV identity.
pub trait TvDiscovery: Send + Sync {
    fn discover(&self) -> SessionFuture<'_, Result<Vec<TvHost>, DiscoveryError>>;
}
