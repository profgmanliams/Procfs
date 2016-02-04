use crate::{from_iter, from_iter_radix, ProcResult};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

use std::io::Read;

/// Syscall information about the process, based on the `/proc/<pid>/syscall` file.
///
/// New variants to this enum may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum Syscall {
    /// The process is running, and so the values are not present
    Running,
    Blocked {
        /// The syscall this process is blocked on.
        /// If the syscall_number is -1, then not blocked on a syscall (blocked for another reason).
        /// Note that the rest of the values are still filled in.
        syscall_number: i64,
        /// The argument registers
        argument_registers: [u64; 6],
        /// e.g. rsp
        stack_pointer: u64,
