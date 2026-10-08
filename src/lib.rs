pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use application::SendRemoteAction;
pub use application::State;
pub use domain::{DeviceDisplay, DeviceId, RemoteAction};
