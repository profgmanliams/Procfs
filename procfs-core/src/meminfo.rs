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
    ///
    /// (since Linux 2.6.28)
    pub inactive_file: Option<u64>,
    /// [To be documented.]
    ///
    /// (From Linux 2.6.28 to 2.6.30, CONFIG_UNEVICTABLE_LRU was required.)
    pub unevictable: Option<u64>,
    /// [To be documented.]
    ///
    /// (From Linux 2.6.28 to 2.6.30, CONFIG_UNEVICTABLE_LRU was required.)
    pub mlocked: Option<u64>,
    /// Total amount of highmem.
    ///
    /// Highmem is all memory above ~860MB of physical  memory.  Highmem areas are for use by
    /// user-space programs, or for the page cache.  The kernel must use tricks to access this
    /// memory, making it slower to access than lowmem.
    ///
    /// (Starting with Linux 2.6.19, CONFIG_HIGHMEM is required.)
    pub high_total: Option<u64>,
    /// Amount of free highmem.
    ///
    /// (Starting with Linux 2.6.19, CONFIG_HIGHMEM is required.)
    pub high_free: Option<u64>,
    /// Total amount of lowmem.
    ///
    /// Lowmem is memory which can be used for every thing  that highmem can be used for,
    /// but it is also available for the kernel's use for its own data structures.
    /// Among many other things, it is where everything from Slab is allocated.
    /// Bad things happen when you're out of lowmem.
    ///
    /// (Starting with Linux 2.6.19, CONFIG_HIGHMEM is required.)
    pub low_total: Option<u64>,
    /// Amount of free lowmem.
    ///
    /// (Starting with Linux 2.6.19, CONFIG_HIGHMEM is required.)
    pub low_free: Option<u64>,
    /// [To be documented.]
    ///
    /// (since Linux 2.6.29.  CONFIG_MMU is required.)
    pub mmap_copy: Option<u64>,
    /// Total amount of swap space available.
    pub swap_total: u64,
    /// Amount of swap space that is currently unused.
    pub swap_free: u64,
    /// Memory which is waiting to get written back to the disk.
    pub dirty: u64,
    /// Memory which is actively being written back to the disk.
    pub writeback: u64,
    /// Non-file backed pages mapped into user-space page tables.
    ///
    /// (since Linux 2.6.18)
    pub anon_pages: Option<u64>,
    /// Files which have been mapped into memory (with mmap(2)), such as libraries.
    pub mapped: u64,
