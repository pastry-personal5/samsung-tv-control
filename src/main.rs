use std::sync::Arc;

use samsung_tv_remote::application::tv_control_coordinator::AppServices;
use samsung_tv_remote::application::tv_setup_service::TvSetupService;
use samsung_tv_remote::infrastructure::macos::keychain::KeychainCredentialStore;
use samsung_tv_remote::infrastructure::preferences::{
    default_app_data_dir, LocalCertificateTrustStore, LocalDeviceRepository,
};
use samsung_tv_remote::infrastructure::samsung::session::SamsungGateway;
use samsung_tv_remote::infrastructure::ssdp_discovery::SsdpDiscovery;
use samsung_tv_remote::presentation::iced::app;

fn main() -> iced::Result {
    let setup = default_app_data_dir().map(|directory| {
        Arc::new(TvSetupService::new(
            LocalDeviceRepository::new(&directory),
            KeychainCredentialStore,
            LocalCertificateTrustStore::new(&directory),
        )) as Arc<dyn samsung_tv_remote::application::tv_setup_service::TvSetupPort>
    });
    app::run(AppServices::new(
        setup,
        Arc::new(SsdpDiscovery),
        Arc::new(SamsungGateway),
    ))
}
