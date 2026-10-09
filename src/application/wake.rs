use crate::domain::MacAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeInterface {
    Wired,
    WiFi,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WakeConfiguration {
    pub wired: Option<MacAddress>,
    pub wifi: Option<MacAddress>,
    pub active: Option<WakeInterface>,
}

impl WakeConfiguration {
    pub fn active_mac(self) -> Option<MacAddress> {
        match self.active? {
            WakeInterface::Wired => self.wired,
            WakeInterface::WiFi => self.wifi,
        }
    }
}
