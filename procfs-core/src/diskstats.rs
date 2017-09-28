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

