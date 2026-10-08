pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use application::ControlState;
pub use application::SendRemoteAction;
pub use domain::{DeviceDisplay, DeviceId, RemoteAction};
