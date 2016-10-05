use super::{expect, from_str, ProcResult};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io};

fn convert_to_bytes(num: u64, unit: &str) -> ProcResult<u64> {
    match unit {
        "B" => Ok(num),
        "KiB" | "kiB" | "kB" | "KB" => Ok(num.saturating_mul(1024)),
        "MiB" | "miB" | "MB" | "mB" => Ok(num.saturating_mul(1024 * 1024)),
        "GiB" | "giB" | "GB" | "gB" => Ok(num.saturating_mul(1024 * 1024 * 1024)),
        unknown => Err(build_internal_error!(format!("Unknown unit type {}", unknown))),
    }
}

/// This  struct  reports  statistics about memory usage on the system, based on
/// the `/proc/meminfo` file.
///
/// It is used by `free(1)` to report the amount of free and used memory (both
/// physical  and  swap)  on  the  system  as well as the shared memory and
/// buffers used by the kernel.  Each struct member is generally reported in
/// bytes, but a few are unitless values.
///
/// Except as noted below, all of the fields have been present since at least
/// Linux 2.6.0.  Some fields are optional and are present only if the kernel
/// was configured with various options; those dependencies are noted in the list.
///
/// **Notes**
///
/// While the file shows kilobytes (kB; 1 kB equals 1000 B),
/// it is actually kibibytes (KiB; 1 KiB equals 1024 B).
///
/// All sizes are converted to bytes. Unitless values, like `hugepages_total` are not affected.
///
/// This imprecision in /proc/meminfo is known,
/// but is not corrected due to legacy concerns -
/// programs rely on /proc/meminfo to specify size with the "kB" string.
///
/// New fields to this struct may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[allow(non_snake_case)]
#[non_exhaustive]
pub struct Meminfo {
    /// Total usable RAM (i.e., physical RAM minus a few reserved bits and the kernel binary code).
    pub mem_total: u64,
    /// The sum of [LowFree](#structfield.low_free) + [HighFree](#structfield.high_free).
    pub mem_free: u64,
    /// An estimate of how much memory is available for starting new applications, without swapping.
    ///
    /// (since Linux 3.14)
    pub mem_available: Option<u64>,
    /// Relatively temporary storage for raw disk blocks that shouldn't get tremendously large (20MB or so).
    pub buffers: u64,
    /// In-memory cache for files read from the disk (the page cache).  Doesn't include SwapCached.
    pub cached: u64,
    /// Memory  that  once  was  swapped  out, is swapped back in but still also is in the swap
    /// file.
    ///
    /// (If memory pressure is high, these pages don't need to be swapped out again
    /// because they are already in the swap file.  This saves I/O.)
    pub swap_cached: u64,
    /// Memory that has been used more recently and usually not reclaimed unless absolutely
    /// necessary.
    pub active: u64,
    /// Memory which has been less recently used.  It is more eligible to be reclaimed for other
    /// purposes.
    pub inactive: u64,
    /// [To be documented.]
    ///
    /// (since Linux 2.6.28)
    pub active_anon: Option<u64>,
    /// [To be documented.]
    ///
    /// (since Linux 2.6.28)
    pub inactive_anon: Option<u64>,
    /// [To be documented.]
    ///
    /// (since Linux 2.6.28)
    pub active_file: Option<u64>,
    /// [To be documented.]
