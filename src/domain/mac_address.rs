use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddress([u8; 6]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacAddressError {
    InvalidFormat,
    InvalidAddress,
}

impl MacAddress {
    pub const fn octets(self) -> [u8; 6] {
        self.0
    }
}

impl FromStr for MacAddress {
    type Err = MacAddressError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let separator = if value.as_bytes().get(2) == Some(&b':') {
            ':'
        } else if value.as_bytes().get(2) == Some(&b'-') {
            '-'
        } else {
            return Err(MacAddressError::InvalidFormat);
        };
        if value.len() != 17 {
            return Err(MacAddressError::InvalidFormat);
        }
        let parts = value.split(separator).collect::<Vec<_>>();
        if parts.len() != 6 || parts.iter().any(|part| part.len() != 2) {
            return Err(MacAddressError::InvalidFormat);
        }
        let mut octets = [0; 6];
        for (octet, part) in octets.iter_mut().zip(parts) {
            *octet = u8::from_str_radix(part, 16).map_err(|_| MacAddressError::InvalidFormat)?;
        }
        if octets[0] & 1 != 0 || octets == [0; 6] {
            return Err(MacAddressError::InvalidAddress);
        }
        Ok(Self(octets))
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(formatter, "{a:02X}:{b:02X}:{c:02X}:{d:02X}:{e:02X}:{f:02X}")
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MacAddress([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_unicast_addresses() {
        let mac: MacAddress = "02-ab-CD-00-12-ef".parse().unwrap();
        assert_eq!(mac.to_string(), "02:AB:CD:00:12:EF");
        assert_eq!(mac.octets(), [2, 0xab, 0xcd, 0, 0x12, 0xef]);
    }

    #[test]
    fn rejects_bad_or_non_unicast_addresses() {
        for value in [
            "",
            "02:ab:cd:00:12",
            "02:ab-cd:00:12:ef",
            "GG:00:00:00:00:00",
            "00:00:00:00:00:00",
            "FF:FF:FF:FF:FF:FF",
            "01:00:5E:00:00:01",
        ] {
            assert!(value.parse::<MacAddress>().is_err(), "{value}");
        }
    }
}
