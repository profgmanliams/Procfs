use std::io::BufRead;

use super::ProcResult;
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Device entries under `/proc/devices`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Devices {
    /// Character devices
    pub char_devices: Vec<CharDeviceEntry>,
    /// Block devices, which can be empty if the kernel doesn't support block devices (without `CONFIG_BLOCK`)
    pub block_devices: Vec<BlockDeviceEntry>,
}

/// A character device entry under `/proc/devices`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct CharDeviceEntry {
    /// Device major number
    pub major: u32,
    /// Device name
    pub name: String,
}

/// A block device entry under `/proc/devices`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct BlockDeviceEntry {
    /// Device major number
    pub major: i32,
    /// Device name
    pub name: String,
}

impl super::FromBufRead for Devices {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        enum State {
            Char,
            Block,
        }
        let mut state = State::Char; // Always start with char devices
        let mut devices = Devices {
            char_devices: vec![],
            block_devices: vec![],
