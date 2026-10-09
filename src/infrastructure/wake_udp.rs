use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};

use crate::application::tv_address::TvHost;
use crate::application::wake_transport::{
    magic_packet, WakeFuture, WakePermit, WakeSendError, WakeTransport,
};
use crate::domain::MacAddress;

pub struct UdpWakeTransport;

impl WakeTransport for UdpWakeTransport {
    fn send_once(&self, host: TvHost, mac: MacAddress, permit: WakePermit) -> WakeFuture {
        Box::pin(async move {
            tokio::task::spawn_blocking(move || send_once(&host, mac, permit))
                .await
                .map_err(|_| WakeSendError::SendFailed)?
        })
    }
}

fn send_once(host: &TvHost, mac: MacAddress, permit: WakePermit) -> Result<(), WakeSendError> {
    if !permit.is_current() {
        return Err(WakeSendError::Cancelled);
    }
    let answers = (host.as_str(), 9)
        .to_socket_addrs()
        .map_err(|_| WakeSendError::InvalidTarget)?
        .map(|address| address.ip())
        .collect::<Vec<_>>();
    host.validate_resolved(&answers)
        .map_err(|_| WakeSendError::InvalidTarget)?;
    let mut had_ipv4_route = false;
    for address in answers {
        let IpAddr::V4(target) = address else {
            continue;
        };
        let probe = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(map_io_error)?;
        if let Err(error) = probe.connect((target, 9)) {
            if error.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(WakeSendError::Permission);
            }
            continue;
        }
        let Ok(SocketAddr::V4(local)) = probe.local_addr() else {
            continue;
        };
        had_ipv4_route = true;
        let Some((broadcast, interface)) = broadcast_for(*local.ip(), target)? else {
            continue;
        };
        let socket = UdpSocket::bind((*local.ip(), 0)).map_err(map_io_error)?;
        socket.set_broadcast(true).map_err(map_io_error)?;
        bind_interface(&socket, interface)?;
        let count = permit.send_if_current(|| {
            socket
                .send_to(&magic_packet(mac), (broadcast, 9))
                .map_err(map_io_error)
        })?;
        return if count == 102 {
            Ok(())
        } else {
            Err(WakeSendError::SendFailed)
        };
    }
    Err(if had_ipv4_route {
        WakeSendError::NoBroadcast
    } else {
        WakeSendError::NoIpv4Route
    })
}

fn map_io_error(error: std::io::Error) -> WakeSendError {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        WakeSendError::Permission
    } else {
        WakeSendError::SendFailed
    }
}

fn broadcast_for(
    local: Ipv4Addr,
    target: Ipv4Addr,
) -> Result<Option<(Ipv4Addr, u32)>, WakeSendError> {
    let mut head: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut head) } != 0 {
        return Err(WakeSendError::NoIpv4Route);
    }
    let mut cursor = head;
    let mut result = None;
    while !cursor.is_null() {
        let entry = unsafe { &*cursor };
        if !entry.ifa_addr.is_null()
            && !entry.ifa_netmask.is_null()
            && unsafe { (*entry.ifa_addr).sa_family } as i32 == libc::AF_INET
            && entry.ifa_flags & (libc::IFF_UP as u32) != 0
            && entry.ifa_flags & (libc::IFF_BROADCAST as u32) != 0
        {
            let address = unsafe { &*(entry.ifa_addr as *const libc::sockaddr_in) };
            let mask = unsafe { &*(entry.ifa_netmask as *const libc::sockaddr_in) };
            if Ipv4Addr::from(address.sin_addr.s_addr.to_ne_bytes()) == local {
                let mask = u32::from_be_bytes(mask.sin_addr.s_addr.to_ne_bytes());
                if let Some(broadcast) = directed_broadcast(local, target, mask) {
                    let index = unsafe { libc::if_nametoindex(entry.ifa_name) };
                    if index != 0 {
                        result = Some((broadcast, index));
                        break;
                    }
                }
            }
        }
        cursor = entry.ifa_next;
    }
    unsafe { libc::freeifaddrs(head) };
    Ok(result)
}

fn directed_broadcast(local: Ipv4Addr, target: Ipv4Addr, mask: u32) -> Option<Ipv4Addr> {
    if mask.count_ones() >= 31 || mask.leading_ones() != mask.count_ones() {
        return None;
    }
    let local = u32::from_be_bytes(local.octets());
    let target = u32::from_be_bytes(target.octets());
    let network = local & mask;
    let broadcast = network | !mask;
    if target == local
        || target & mask != network
        || target == network
        || target == broadcast
        || broadcast == u32::MAX
    {
        return None;
    }
    Some(Ipv4Addr::from(broadcast))
}

#[cfg(target_os = "macos")]
fn bind_interface(socket: &UdpSocket, interface: u32) -> Result<(), WakeSendError> {
    use std::os::fd::AsRawFd;
    let result = unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::IPPROTO_IP,
            libc::IP_BOUND_IF,
            &interface as *const u32 as *const libc::c_void,
            std::mem::size_of::<u32>() as libc::socklen_t,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(map_io_error(std::io::Error::last_os_error()))
    }
}

#[cfg(not(target_os = "macos"))]
fn bind_interface(_: &UdpSocket, _: u32) -> Result<(), WakeSendError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directed_broadcast_requires_same_usable_subnet() {
        let local = Ipv4Addr::new(192, 168, 10, 4);
        assert_eq!(
            directed_broadcast(local, Ipv4Addr::new(192, 168, 10, 20), 0xffffff00),
            Some(Ipv4Addr::new(192, 168, 10, 255))
        );
        assert_eq!(
            directed_broadcast(local, Ipv4Addr::new(192, 168, 11, 20), 0xffffff00),
            None
        );
        assert_eq!(
            directed_broadcast(local, Ipv4Addr::new(192, 168, 10, 5), 0xfffffffe),
            None
        );
        assert_eq!(
            directed_broadcast(local, Ipv4Addr::new(192, 168, 10, 5), 0xff00ff00),
            None
        );
    }
}
