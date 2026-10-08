use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::{timeout, Instant};

use crate::application::discovery::{DeviceDiscovery, DiscoveryError};
use crate::application::target::{is_local_tv_address, TvHost};

const SEARCH: &[u8] = b"M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: ssdp:all\r\n\r\n";
const SEARCH_WINDOW: Duration = Duration::from_secs(5);
const MAX_CANDIDATES: usize = 32;

#[derive(Debug, Default)]
pub struct SsdpDiscovery;

impl DeviceDiscovery for SsdpDiscovery {
    async fn discover(&self) -> Result<Vec<TvHost>, DiscoveryError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(classify_error)?;
        socket
            .send_to(SEARCH, "239.255.255.250:1900")
            .await
            .map_err(classify_error)?;
        let deadline = Instant::now() + SEARCH_WINDOW;
        let mut candidates = Vec::new();
        let mut buffer = [0u8; 2049];
        while candidates.len() < MAX_CANDIDATES {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            let (length, peer) = match timeout(remaining, socket.recv_from(&mut buffer)).await {
                Ok(Ok(response)) => response,
                Ok(Err(error)) => return Err(classify_error(error)),
                Err(_) => break,
            };
            if length > 2048 {
                continue;
            }
            if let Some(host) = candidate_from_response(&buffer[..length], peer) {
                if !candidates.contains(&host) {
                    candidates.push(host);
                }
            }
        }
        Ok(candidates)
    }
}

fn classify_error(error: std::io::Error) -> DiscoveryError {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => DiscoveryError::Permission,
        _ => DiscoveryError::Unavailable,
    }
}

fn candidate_from_response(response: &[u8], peer: SocketAddr) -> Option<TvHost> {
    if !is_local_tv_address(peer.ip()) || response.len() > 2048 {
        return None;
    }
    let response = std::str::from_utf8(response).ok()?;
    let mut lines = response.split("\r\n");
    if !lines.next()?.starts_with("HTTP/1.1 200") {
        return None;
    }
    let samsung = lines.any(|line| {
        let lower = line.to_ascii_lowercase();
        (lower.starts_with("server:") || lower.starts_with("st:") || lower.starts_with("usn:"))
            && lower.contains("samsung")
    });
    if !samsung {
        return None;
    }
    match peer.ip() {
        IpAddr::V4(address) => TvHost::parse(&address.to_string()).ok(),
        IpAddr::V6(address) => TvHost::parse(&address.to_string()).ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_only_samsung_responses_from_private_sources() {
        let response = b"HTTP/1.1 200 OK\r\nSERVER: Samsung UPnP/1.0\r\nLOCATION: http://public.example/\r\n\r\n";
        let local = SocketAddr::from(([192, 168, 1, 2], 1900));
        assert_eq!(
            candidate_from_response(response, local),
            TvHost::parse("192.168.1.2").ok()
        );
        assert!(
            candidate_from_response(response, SocketAddr::from(([8, 8, 8, 8], 1900))).is_none()
        );
        assert!(candidate_from_response(b"HTTP/1.1 200 OK\r\nSERVER: Other\r\n", local).is_none());
    }

    #[test]
    fn permission_errors_remain_distinct_from_empty_discovery() {
        assert_eq!(
            classify_error(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
            DiscoveryError::Permission
        );
        assert_eq!(
            classify_error(std::io::Error::from(std::io::ErrorKind::NetworkUnreachable)),
            DiscoveryError::Unavailable
        );
    }
}
