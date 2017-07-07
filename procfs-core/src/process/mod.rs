//! Functions and structs related to process information
//!
//! The primary source of data for functions in this module is the files in a `/proc/<pid>/`
//! directory.

use super::*;
use crate::from_iter;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

mod limit;
pub use limit::*;

mod stat;
pub use stat::*;

mod mount;
pub use mount::*;

mod namespaces;
pub use namespaces::*;

mod status;
pub use status::*;

mod schedstat;
pub use schedstat::*;

mod smaps_rollup;
pub use smaps_rollup::*;

mod syscall;
pub use syscall::*;

mod pagemap;
pub use pagemap::*;

mod clear_refs;
pub use clear_refs::*;

bitflags! {
    /// Kernel flags for a process
    ///
    /// See also the [Stat::flags()] method.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct StatFlags: u32 {
        /// I am an IDLE thread
        const PF_IDLE = 0x0000_0002;
        /// Getting shut down
        const PF_EXITING = 0x0000_0004;
        /// PI exit done on shut down
        const PF_EXITPIDONE = 0x0000_0008;
        /// I'm a virtual CPU
        const PF_VCPU = 0x0000_0010;
        /// I'm a workqueue worker
        const PF_WQ_WORKER = 0x0000_0020;
        /// Forked but didn't exec
        const PF_FORKNOEXEC = 0x0000_0040;
        /// Process policy on mce errors;
        const PF_MCE_PROCESS = 0x0000_0080;
        /// Used super-user privileges
        const PF_SUPERPRIV = 0x0000_0100;
        /// Dumped core
        const PF_DUMPCORE = 0x0000_0200;
        /// Killed by a signal
        const PF_SIGNALED = 0x0000_0400;
        ///Allocating memory
        const PF_MEMALLOC = 0x0000_0800;
        /// set_user() noticed that RLIMIT_NPROC was exceeded
        const PF_NPROC_EXCEEDED = 0x0000_1000;
        /// If unset the fpu must be initialized before use
        const PF_USED_MATH = 0x0000_2000;
         /// Used async_schedule*(), used by module init
        const PF_USED_ASYNC = 0x0000_4000;
        ///  This thread should not be frozen
        const PF_NOFREEZE = 0x0000_8000;
        /// Frozen for system suspend
        const PF_FROZEN = 0x0001_0000;
        /// I am kswapd
        const PF_KSWAPD = 0x0002_0000;
        /// All allocation requests will inherit GFP_NOFS
        const PF_MEMALLOC_NOFS = 0x0004_0000;
        /// All allocation requests will inherit GFP_NOIO
        const PF_MEMALLOC_NOIO = 0x0008_0000;
        /// Throttle me less: I clean memory
        const PF_LESS_THROTTLE = 0x0010_0000;
        /// I am a kernel thread
        const PF_KTHREAD = 0x0020_0000;
        /// Randomize virtual address space
        const PF_RANDOMIZE = 0x0040_0000;
        /// Allowed to write to swap
        const PF_SWAPWRITE = 0x0080_0000;
        /// Stalled due to lack of memory
        const PF_MEMSTALL = 0x0100_0000;
        /// I'm an Usermodehelper process
        const PF_UMH = 0x0200_0000;
        /// Userland is not allowed to meddle with cpus_allowed
        const PF_NO_SETAFFINITY = 0x0400_0000;
        /// Early kill for mce process policy
        const PF_MCE_EARLY = 0x0800_0000;
        /// All allocation request will have _GFP_MOVABLE cleared
        const PF_MEMALLOC_NOCMA = 0x1000_0000;
        /// Thread belongs to the rt mutex tester
        const PF_MUTEX_TESTER = 0x2000_0000;
        /// Freezer should not count it as freezable
        const PF_FREEZER_SKIP = 0x4000_0000;
        /// This thread called freeze_processes() and should not be frozen
        const PF_SUSPEND_TASK = 0x8000_0000;

    }
}

bitflags! {
    /// See the [coredump_filter()](struct.Process.html#method.coredump_filter) method.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct CoredumpFlags: u32 {
        const ANONYMOUS_PRIVATE_MAPPINGS = 0x01;
        const ANONYMOUS_SHARED_MAPPINGS = 0x02;
        const FILEBACKED_PRIVATE_MAPPINGS = 0x04;
        const FILEBACKED_SHARED_MAPPINGS = 0x08;
        const ELF_HEADERS = 0x10;
        const PROVATE_HUGEPAGES = 0x20;
        const SHARED_HUGEPAGES = 0x40;
        const PRIVATE_DAX_PAGES = 0x80;
        const SHARED_DAX_PAGES = 0x100;
    }
}

bitflags! {
    /// The permissions a process has on memory map entries.
    ///
    /// Note that the `SHARED` and `PRIVATE` are mutually exclusive, so while you can
    /// use `MMPermissions::all()` to construct an instance that has all bits set,
    /// this particular value would never been seen in procfs.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
    pub struct MMPermissions: u8 {
        /// No permissions
        const NONE = 0;
        /// Read permission
        const READ = 1 << 0;
        /// Write permission
        const WRITE = 1 << 1;
        /// Execute permission
        const EXECUTE = 1 << 2;
        /// Memory is shared with another process.
        ///
        /// Mutually exclusive with PRIVATE.
        const SHARED = 1 << 3;
        /// Memory is private (and copy-on-write)
        ///
        /// Mutually exclusive with SHARED.
        const PRIVATE = 1 << 4;
    }
}

impl MMPermissions {
    fn from_ascii_char(b: u8) -> Self {
        match b {
            b'r' => Self::READ,
            b'w' => Self::WRITE,
            b'x' => Self::EXECUTE,
            b's' => Self::SHARED,
            b'p' => Self::PRIVATE,
            _ => Self::NONE,
        }
    }
    /// Returns this permission map as a 4-character string, similar to what you
    /// might see in `/proc/\<pid\>/maps`.
    ///
    /// Note that the SHARED and PRIVATE bits are mutually exclusive, so this
    /// string is 4 characters long, not 5.
    pub fn as_str(&self) -> String {
        let mut s = String::with_capacity(4);
        s.push(if self.contains(Self::READ) { 'r' } else { '-' });
        s.push(if self.contains(Self::WRITE) { 'w' } else { '-' });
        s.push(if self.contains(Self::EXECUTE) { 'x' } else { '-' });
        s.push(if self.contains(Self::SHARED) {
            's'
        } else if self.contains(Self::PRIVATE) {
            'p'
        } else {
            '-'
        });

        s
    }
}

impl FromStr for MMPermissions {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Only operate on ASCII (byte) values
        Ok(s.bytes()
            .map(Self::from_ascii_char)
            .fold(Self::default(), std::ops::BitOr::bitor))
    }
}

bitflags! {
    /// Represents the kernel flags associated with the virtual memory area.
    /// The names of these flags are just those you'll find in the man page, but in upper case.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Default)]
    pub struct VmFlags: u32 {
        /// No flags
