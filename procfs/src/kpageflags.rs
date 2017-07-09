use crate::{process::Pfn, FileWrapper, ProcResult};

use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    mem::size_of,
    path::Path,
};

pub use procfs_core::PhysicalPageFlags;

/// Parse physical memory flags accessing `/proc/kpageflags`.
///
/// Require root or CAP_SYS_ADMIN
pub struct KPageFlags {
    reader: BufReader<FileWrapper>,
}

impl KPageFlags {
    /// Get a parser from default `/proc/kpageflags`
    ///
    /// Return `Err` if process is not running as root or don't have CAP_SYS_ADMIN
    pub fn new() -> ProcResult<Self> {
        Self::from_custom_root("/proc")
    }

