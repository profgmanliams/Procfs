use crate::{FromStrRadix, ProcResult};
use std::collections::HashMap;
use std::io::BufRead;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Status information about the process, based on the `/proc/<pid>/status` file.
///
/// Not all fields are available in every kernel.  These fields have `Option<T>` types.
/// In general, the current kernel version will tell you what fields you can expect, but this
/// isn't totally reliable, since some kernels might backport certain fields, or fields might
/// only be present if certain kernel configuration options are enabled.  Be prepared to
/// handle `None` values.
///
/// New fields to this struct may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Status {
    /// Command run by this process.
    pub name: String,
    /// Process umask, expressed in octal with a leading zero; see umask(2).  (Since Linux 4.7.)
    pub umask: Option<u32>,
    /// Current state of the process.
    pub state: String,
    /// Thread group ID (i.e., Process ID).
    pub tgid: i32,
    /// NUMA group ID (0 if none; since Linux 3.13).
    pub ngid: Option<i32>,
    /// Thread ID (see gettid(2)).
    pub pid: i32,
    /// PID of parent process.
    pub ppid: i32,
    /// PID of process tracing this process (0 if not being traced).
    pub tracerpid: i32,
    /// Real UID.
    pub ruid: u32,
    /// Effective UID.
    pub euid: u32,
    /// Saved set UID.
    pub suid: u32,
    /// Filesystem UID.
    pub fuid: u32,
    /// Real GID.
    pub rgid: u32,
    /// Effective GID.
    pub egid: u32,
    /// Saved set GID.
    pub sgid: u32,
    /// Filesystem GID.
    pub fgid: u32,
    /// Number of file descriptor slots currently allocated.
    pub fdsize: u32,
    /// Supplementary group list.
    pub groups: Vec<u32>,
    /// Thread group ID (i.e., PID) in each of the PID
    /// namespaces of which (pid)[struct.Status.html#structfield.pid] is a member.  The leftmost entry
    /// shows the value with respect to the PID namespace of the
    /// reading process, followed by the value in successively
    /// nested inner namespaces.  (Since Linux 4.1.)
    pub nstgid: Option<Vec<i32>>,
    /// Thread ID in each of the PID namespaces of which
    /// (pid)[struct.Status.html#structfield.pid] is a member.  The fields are ordered as for NStgid.
    /// (Since Linux 4.1.)
    pub nspid: Option<Vec<i32>>,
    /// Process group ID in each of the PID namespaces of
    /// which (pid)[struct.Status.html#structfield.pid] is a member.  The fields are ordered as for NStgid.  (Since Linux 4.1.)
