//! These files provide additional information about the /dev/random device
//!
//! Note that some of these entries are only documented in random(4), while some are also documented under proc(5)

use crate::{read_value, write_value, ProcError, ProcResult};
use std::path::Path;

const RANDOM_ROOT: &str = "/proc/sys/kernel/random"; 

/// This read-only file gives the available entropy, in bits. This will be a number in the range
/// 0 to 4096
pub fn entropy_avail() -> ProcResult<u16> {
    read_value(Path::new(RANDOM_ROOT).join("entropy_avail"))
}

/// This file gives the size of the entropy pool
///
/// The semantics of this file are different on kernel versions older than 2.6, however, since
/// Linux 2.6 it is read-only, and gives the size of the entropy pool in bits, containing the value 4096.
///
/// See `man random(4)` for more information
pub fn poolsize() -> ProcResult<u16> {
