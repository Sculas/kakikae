use std::fmt::{Debug, Display, Formatter};

#[derive(thiserror::Error)]
pub enum KakikaeError {
    #[error("Failed to convert byte array to an integer: {0:?}")]
    IntConvertFail(Vec<u8>),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Usb error: {0}")]
    Usb(#[from] nusb::Error),
    #[error("Usb ctrl_transfer error: {0}")]
    UsbCtrlTransfer(#[from] nusb::transfer::TransferError),
    #[error("Command fail: {0}")]
    StatusError(u16),
    #[error("Command fail, Sent: {0:X?}, Received {1:X?}")]
    EchoMismatch(Vec<u8>, Vec<u8>),
    #[error("Failed to handshake for over 5 seconds.")]
    HandshakeTimeout,
    #[error("This utility only supports MT6785 at the moment.")]
    UnsupportedDevice,
    #[error("Failed to find stage file: '{0}', Did you follow the steps correctly?")]
    StageDataNotFound(String),
}
impl Debug for KakikaeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}