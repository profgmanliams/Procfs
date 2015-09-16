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
    /// Number of 1024 byte blocks
    pub blocks: u64,
    /// Device name
    pub name: String,
}

impl super::FromBufRead for Vec<PartitionEntry> {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        for line in r.lines().skip(2) {
            let line = expect!(line);

            let mut s = line.split_whitespace();

            let major = expect!(u16::from_str(expect!(s.next())));
            let minor = expect!(u16::from_str(expect!(s.next())));
            let blocks = expect!(u64::from_str(expect!(s.next())));
            let name = expect!(s.next()).to_string();

            let partition_entry = PartitionEntry {
                major,
                minor,
                blocks,
                name,
            };
