use super::target::TvHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    Permission,
    Unavailable,
}

/// Discovery results are untrusted hints and never establish TV identity.
pub trait DeviceDiscovery {
    fn discover(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TvHost>, DiscoveryError>> + Send;
}
