use std::{collections::HashMap, io::BufRead};

use super::ProcResult;
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// A mountpoint entry under `/proc/mounts`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[allow(non_snake_case)]
pub struct MountEntry {
    /// Device
    pub fs_spec: String,
    /// Mountpoint
    pub fs_file: String,
    /// FS type
    pub fs_vfstype: String,
    /// Mount options
    pub fs_mntops: HashMap<String, Option<String>>,
    /// Dump
    pub fs_freq: u8,
    /// Check
    pub fs_passno: u8,
}

impl super::FromBufRead for Vec<MountEntry> {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
