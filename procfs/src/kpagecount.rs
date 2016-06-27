use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    mem::size_of,
    path::Path,
};

use crate::{process::Pfn, FileWrapper};

use super::ProcResult;

/// Parse physical memory references accessing `/proc/kpagecount`
///
/// Require root or CAP_SYS_ADMIN
pub struct KPageCount {
    reader: BufReader<FileWrapper>,
}

impl KPageCount {
    /// Get a parser from default `/proc/kpagecount`
    ///
    /// Return `Err` if process is not running as root or don't have CAP_SYS_ADMIN
    pub fn new() -> ProcResult<Self> {
        Self::from_custom_root("/proc")
    }

    /// Get a parser from custom `/proc`
    ///
    /// Return `Err` if process is not running as root or don't have CAP_SYS_ADMIN
