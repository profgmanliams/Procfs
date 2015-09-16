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
    /// Number of attached processes
    pub nattch: u32,
    /// User ID
    pub uid: u16,
    /// Group ID
    pub gid: u16,
    /// Creator UID
    pub cuid: u16,
    /// Creator GID
    pub cgid: u16,
    /// Time of last `shmat` (attach), epoch
    pub atime: u64,
    /// Time of last `shmdt` (detach), epoch
    pub dtime: u64,
    /// Time of last permission change, epoch
    pub ctime: u64,
    /// Current part of the shared memory resident in memory
    pub rss: u64,
    /// Current part of the shared memory in SWAP
    pub swap: u64,
}

/// A set of shared memory segments parsed from `/proc/sysvipc/shm`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct SharedMemorySegments(pub Vec<Shm>);
