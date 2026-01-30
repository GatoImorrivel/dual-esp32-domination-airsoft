pub enum BtEvent {
    ListDevices,
    ConnectDevice,
    DisconnectDevice,
    SendAudioBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtDevice {
    pub name: Option<String>,
    pub addr: [u8; 6]
}

#[derive(Debug, Clone)]
pub struct ListDevicesResponse {
    devices: [BtDevice; 10]
}