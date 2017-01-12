//! Memory management tuning buffer and cache management
//!
//! The files in this directory can be used to tune
//! the operation of the virtual memory (VM) subsystem of the Linux kernel
//! and the write out of dirty data to disk.

use std::fmt;
use std::str;

use crate::{read_value, write_value, ProcResult};

/// The amount of free memory in the system that should be reserved for users with the capability cap_sys_admin.
///
/// # Example
///
/// ```
/// use procfs::sys::vm::admin_reserve_kbytes;
///
/// assert_ne!(admin_reserve_kbytes().unwrap(), 0);
/// ```
pub fn admin_reserve_kbytes() -> ProcResult<usize> {
    read_value("/proc/sys/vm/admin_reserve_kbytes")
}

/// Set the amount of free memory in the system that should be reserved for users with the capability cap_sys_admin.
pub fn set_admin_reserve_kbytes(kbytes: usize) -> ProcResult<()> {
    write_value("/proc/sys/vm/admin_reserve_kbytes", kbytes)
}

/// Force all zones are compacted such that free memory is available in contiguous blocks where possible.
///
/// This can be important for example in the allocation of huge pages
/// although processes will also directly compact memory as required.
///
/// Present only if the kernel was configured with CONFIG_COMPACTION.
