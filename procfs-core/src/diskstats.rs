use crate::{expect, from_str, ProcResult};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::io::BufRead;

/// Disk IO stat information
///
/// To fully understand these fields, please see the [iostats.txt](https://www.kernel.org/doc/Documentation/iostats.txt)
/// kernel documentation.
///
/// For an example, see the [diskstats.rs](https://github.com/eminence/procfs/tree/master/procfs/examples)
/// example in the source repo.
// Doc reference: https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats
// Doc reference: https://www.kernel.org/doc/Documentation/iostats.txt
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct DiskStat {
    /// The device major number
    pub major: i32,

    /// The device minor number
    pub minor: i32,

    /// Device name
    pub name: String,

    /// Reads completed successfully
    ///
    /// This is the total number of reads completed successfully
    pub reads: u64,

    /// Reads merged
    ///
    /// The number of adjacent reads that have been merged for efficiency.
    pub merged: u64,

    /// Sectors read successfully
    ///
    /// This is the total number of sectors read successfully.
    pub sectors_read: u64,

    /// Time spent reading (ms)
    pub time_reading: u64,

    /// writes completed
    pub writes: u64,

    /// writes merged
    ///
    /// The number of adjacent writes that have been merged for efficiency.
    pub writes_merged: u64,

    /// Sectors written successfully
    pub sectors_written: u64,

    /// Time spent writing (ms)
    pub time_writing: u64,

    /// I/Os currently in progress
    pub in_progress: u64,

    /// Time spent doing I/Os (ms)
    pub time_in_progress: u64,

    /// Weighted time spent doing I/Os (ms)
    pub weighted_time_in_progress: u64,

    /// Discards completed successfully
    ///
    /// (since kernel 4.18)
    pub discards: Option<u64>,

    /// Discards merged
    pub discards_merged: Option<u64>,

    /// Sectors discarded
    pub sectors_discarded: Option<u64>,

    /// Time spent discarding
    pub time_discarding: Option<u64>,

    /// Flush requests completed successfully
    ///
    /// (since kernel 5.5)
    pub flushes: Option<u64>,

    /// Time spent flushing
    pub time_flushing: Option<u64>,
}

/// A list of disk stats.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct DiskStats(pub Vec<DiskStat>);

impl crate::FromBufRead for DiskStats {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut v = Vec::new();

        for line in r.lines() {
            let line = line?;
            v.push(DiskStat::from_line(&line)?);
        }
        Ok(DiskStats(v))
