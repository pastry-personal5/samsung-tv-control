use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use super::tv_address::TvHost;
use crate::domain::MacAddress;

pub type WakeFuture = Pin<Box<dyn Future<Output = Result<(), WakeSendError>> + Send>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeSendError {
    Cancelled,
    InvalidTarget,
    NoIpv4Route,
    NoBroadcast,
    Permission,
    SendFailed,
}

pub trait WakeTransport: Send + Sync {
    fn send_once(&self, host: TvHost, mac: MacAddress, permit: WakePermit) -> WakeFuture;
}

#[derive(Clone)]
pub struct WakePermit {
    pub(crate) epoch: Arc<AtomicU64>,
    pub(crate) attempt: u64,
    pub(crate) send_guard: Arc<Mutex<()>>,
}

impl WakePermit {
    pub fn is_current(&self) -> bool {
        self.epoch.load(Ordering::SeqCst) == self.attempt
    }

    /// Cancellation takes the same guard, so a cancelled attempt cannot start
    /// its final datagram write after cancellation returns.
    pub fn send_if_current<T>(
        &self,
        send: impl FnOnce() -> Result<T, WakeSendError>,
    ) -> Result<T, WakeSendError> {
        let _guard = self
            .send_guard
            .lock()
            .map_err(|_| WakeSendError::Cancelled)?;
        if !self.is_current() {
            return Err(WakeSendError::Cancelled);
        }
        send()
    }
}

pub fn magic_packet(mac: MacAddress) -> [u8; 102] {
    let mut packet = [0; 102];
    packet[..6].fill(0xff);
    for index in 0..16 {
        let start = 6 + index * 6;
        packet[start..start + 6].copy_from_slice(&mac.octets());
    }
    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_contains_six_sync_bytes_and_sixteen_mac_copies() {
        let mac = "02:11:22:33:44:55".parse().unwrap();
        let packet = magic_packet(mac);
        assert_eq!(&packet[..6], &[0xff; 6]);
        assert!(packet[6..]
            .as_chunks::<6>()
            .0
            .iter()
            .all(|part| *part == mac.octets()));
    }

    #[test]
    fn cancelled_permit_never_enters_the_send_closure() {
        let epoch = Arc::new(AtomicU64::new(1));
        let guard = Arc::new(Mutex::new(()));
        let permit = WakePermit {
            epoch: epoch.clone(),
            attempt: 1,
            send_guard: guard.clone(),
        };
        {
            let _held = guard.lock().unwrap();
            epoch.store(2, Ordering::SeqCst);
        }
        assert_eq!(
            permit.send_if_current::<()>(|| panic!("send after cancellation")),
            Err(WakeSendError::Cancelled)
        );
    }
}
