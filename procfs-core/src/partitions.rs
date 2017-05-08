use std::io::BufRead;

use super::ProcResult;
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// A partition entry under `/proc/partitions`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[allow(non_snake_case)]
pub struct PartitionEntry {
    /// Device major number
    pub major: u16,
    /// Device minor number
    pub minor: u16,
