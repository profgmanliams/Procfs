//! This modules contains functions for kernel variables related to filesystems

use crate::{expect, from_str, read_file, read_value, write_value, ProcResult};
use std::time::Duration;

pub mod binfmt_misc;
pub mod epoll;

/// Information about the status of the directory cache (dcache)
#[derive(Debug, Clone)]
pub struct DEntryState {
    /// The number of allocated dentries (dcache entries)
    ///
    /// Unused in Linux 2.2
    pub nr_dentry: u32,

    /// The number of unused dentries.
    pub nr_unused: u32,

    /// The age after which dcache entries can be reclaimied when memory is short
    pub age_limit: Duration,

    /// Is true when the kernel has called `shrink_dcache_pages()` and the dcache isn't pruned yet.
    pub want_pages: bool,
}

impl DEntryState {
    fn from_str(s: &str) -> ProcResult<DEntryState> {
        let mut s = s.split_whitespace();
        let nr_dentry = from_str!(u32, expect!(s.next()));
        let nr_unused = from_str!(u32, expect!(s.next()));
        let age_limit_sec = from_str!(u32, expect!(s.next()));
        let want_pages = from_str!(u32, expect!(s.next()));

