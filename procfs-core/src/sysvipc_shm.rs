use std::io;

use super::{expect, ProcResult};
use std::str::FromStr;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// A shared memory segment parsed from `/proc/sysvipc/shm`
/// Relation with [crate::process::MMapPath::Vsys]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[allow(non_snake_case)]
pub struct Shm {
    /// Segment key
    pub key: i32,
    /// Segment ID, unique
    pub shmid: u64,
    /// Access permissions, as octal
    pub perms: u16,
    /// Size in bytes
    pub size: u64,
    /// Creator PID
    pub cpid: i32,
    /// Last operator PID
    pub lpid: i32,
