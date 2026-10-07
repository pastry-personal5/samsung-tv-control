pub mod application;
pub mod domain;
pub mod presentation;

pub use application::State;
pub use application::SendRemoteAction;
pub use domain::{DeviceId, RemoteAction, DeviceDisplay};
