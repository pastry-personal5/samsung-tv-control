use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The host portion of a local TV target. A scheme, port, path, or credentials
/// are never accepted from a settings field.
#[derive(Clone, PartialEq, Eq)]
pub struct TvHost(String);

impl std::fmt::Debug for TvHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TvHost([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetError {
    InvalidHost,
    NonLocalAddress,
    NoResolvedAddress,
}

impl TvHost {
    pub fn parse(input: &str) -> Result<Self, TargetError> {
        if input.is_empty() || input.len() > 253 || input.trim() != input {
            return Err(TargetError::InvalidHost);
        }
        if let Ok(address) = input.parse::<IpAddr>() {
            return if is_local_tv_address(address) {
                Ok(Self(input.to_owned()))
            } else {
                Err(TargetError::NonLocalAddress)
            };
        }
        if input.contains(':') || input.contains('/') || input.contains('@') || input.contains('?')
        {
            return Err(TargetError::InvalidHost);
        }
        let host = input.strip_suffix('.').unwrap_or(input);
        if host.is_empty()
            || host.split('.').any(|label| {
                label.is_empty()
                    || label.len() > 63
                    || label.starts_with('-')
                    || label.ends_with('-')
                    || !label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            })
        {
            return Err(TargetError::InvalidHost);
        }
        // A numeric-looking host must parse as an address. This prevents an
        // alternate textual IPv4 form from bypassing the address policy.
        if host
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
        {
            return Err(TargetError::InvalidHost);
        }
        Ok(Self(host.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Call after every DNS lookup, including reconnect; reject mixed public
    /// and private answers rather than choosing a convenient first address.
    pub fn validate_resolved(&self, addresses: &[IpAddr]) -> Result<(), TargetError> {
        if addresses.is_empty() {
            return Err(TargetError::NoResolvedAddress);
        }
        if addresses
            .iter()
            .any(|address| !is_local_tv_address(*address))
        {
            return Err(TargetError::NonLocalAddress);
        }
        if let Ok(literal) = self.0.parse::<IpAddr>() {
            if addresses.iter().any(|address| *address != literal) {
                return Err(TargetError::NonLocalAddress);
            }
        }
        Ok(())
    }
}

pub fn is_local_tv_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_local_v4(address),
        IpAddr::V6(address) => is_local_v6(address),
    }
}

fn is_local_v4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    address.is_private() || (octets[0] == 169 && octets[1] == 254 && octets[2] != 0)
}

fn is_local_v6(address: Ipv6Addr) -> bool {
    let first = address.segments()[0];
    // fc00::/7 (unique local) or fe80::/10 (link local).
    (first & 0xfe00 == 0xfc00) || (first & 0xffc0 == 0xfe80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_local_literals_and_valid_dns_hosts() {
        for input in [
            "192.168.1.2",
            "10.4.3.2",
            "172.16.0.2",
            "fe80::1234",
            "tv.local",
        ] {
            assert!(TvHost::parse(input).is_ok(), "{input}");
        }
    }

    #[test]
    fn rejects_urls_ports_public_and_loopback_targets() {
        for input in [
            "https://tv.local",
            "tv.local:8001",
            "tv.local/path",
            "user@tv.local",
            "127.0.0.1",
            "8.8.8.8",
            "::1",
            "224.0.0.1",
            "192.168.1.2 ",
            "3232235778",
            "192.168.1.999",
        ] {
            assert!(TvHost::parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn rejects_any_public_or_changed_dns_answer() {
        let host = TvHost::parse("tv.local").unwrap();
        assert_eq!(
            host.validate_resolved(&[]),
            Err(TargetError::NoResolvedAddress)
        );
        assert!(host
            .validate_resolved(&["192.168.1.2".parse().unwrap()])
            .is_ok());
        assert_eq!(
            host.validate_resolved(&["192.168.1.2".parse().unwrap(), "8.8.8.8".parse().unwrap(),]),
            Err(TargetError::NonLocalAddress)
        );
        let literal = TvHost::parse("192.168.1.2").unwrap();
        assert_eq!(
            literal.validate_resolved(&["192.168.1.3".parse().unwrap()]),
            Err(TargetError::NonLocalAddress)
        );
    }
}
