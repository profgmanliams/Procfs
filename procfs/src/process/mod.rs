//! Functions and structs related to process information
//!
//! The primary source of data for functions in this module is the files in a `/proc/<pid>/`
//! directory.  If you have a process ID, you can use
//! [`Process::new(pid)`](struct.Process.html#method.new), otherwise you can get a
//! list of all running processes using [`all_processes()`](fn.all_processes.html).
//!
//! In case you have procfs filesystem mounted to a location other than `/proc`,
//! use [`Process::new_with_root()`](struct.Process.html#method.new_with_root).
//!
//! # Examples
//!
//! Here's a small example that prints out all processes that are running on the same tty as the calling
//! process.  This is very similar to what "ps" does in its default mode.  You can run this example
//! yourself with:
//!
//! > cargo run --example=ps
//!
//! ```rust
//! let me = procfs::process::Process::myself().unwrap();
//! let me_stat = me.stat().unwrap();
//! let tps = procfs::ticks_per_second();
//!
//! println!("{: >10} {: <8} {: >8} {}", "PID", "TTY", "TIME", "CMD");
//!
//! let tty = format!("pty/{}", me_stat.tty_nr().1);
//! for prc in procfs::process::all_processes().unwrap() {
//!     let Ok(prc) = prc else {
//!         // process vanished
//!         continue;
//!     };
//!     let Ok(stat) = prc.stat() else {
//!         // process vanished
//!         continue;
//!     };
//!     if let Ok(stat) = prc.stat() {
//!         if stat.tty_nr == me_stat.tty_nr {
//!             // total_time is in seconds
//!             let total_time =
//!                 (stat.utime + stat.stime) as f32 / (tps as f32);
//!             println!(
//!                 "{: >10} {: <8} {: >8} {}",
//!                 stat.pid, tty, total_time, stat.comm
//!             );
//!         }
//!     }
//! }
//! ```
//!
//! Here's a simple example of how you could get the total memory used by the current process.
//! There are several ways to do this.  For a longer example, see the `examples/self_memory.rs`
//! file in the git repository.  You can run this example with:
//!
//! > cargo run --example=self_memory
//!
//! ```rust
//! # use procfs::process::Process;
//! let me = Process::myself().unwrap();
//! let me_stat = me.stat().unwrap();
//! let page_size = procfs::page_size();
//!
//! println!("== Data from /proc/self/stat:");
//! println!("Total virtual memory used: {} bytes", me_stat.vsize);
//! println!("Total resident set: {} pages ({} bytes)", me_stat.rss, me_stat.rss as u64 * page_size);
//! ```

use super::*;
use crate::net::{TcpNetEntry, UdpNetEntry};
use crate::sys::kernel::Version;

pub use procfs_core::process::*;
use rustix::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
use rustix::fs::{AtFlags, Mode, OFlags, RawMode};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::read_link;
use std::io::{self, Read};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::str::FromStr;

mod namespaces;

mod task;
pub use task::*;

mod pagemap;
pub use pagemap::*;

#[cfg(test)]
mod tests;

bitflags! {
    /// The mode (read/write permissions) for an open file descriptor
    ///
    /// This is represented as `u16` since the values of these bits are
    /// [documented] to be within the `u16` range.
    ///
    /// [documented]: https://man7.org/linux/man-pages/man2/chmod.2.html#DESCRIPTION
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct FDPermissions: u16 {
        const READ = Mode::RUSR.bits() as u16;
        const WRITE = Mode::WUSR.bits() as u16;
        const EXECUTE = Mode::XUSR.bits() as u16;
    }
}

/// See the [Process::fd()] method
#[derive(Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct FDInfo {
    /// The file descriptor
    pub fd: i32,
    /// The permission bits for this FD
    ///
    /// **Note**: this field is only the owner read/write/execute bits.  All the other bits
    /// (include filetype bits) are masked out.  See also the `mode()` method.
    pub mode: u16,
    pub target: FDTarget,
}

impl FDInfo {
    /// Gets a file descriptor from a raw fd
    pub fn from_raw_fd(pid: i32, raw_fd: i32) -> ProcResult<Self> {
        Self::from_raw_fd_with_root("/proc", pid, raw_fd)
    }

    /// Gets a file descriptor from a raw fd based on a specified `/proc` path
    pub fn from_raw_fd_with_root(root: impl AsRef<Path>, pid: i32, raw_fd: i32) -> ProcResult<Self> {
        let path = root.as_ref().join(pid.to_string()).join("fd").join(raw_fd.to_string());
        let link = wrap_io_error!(path, read_link(&path))?;
        let md = wrap_io_error!(path, path.symlink_metadata())?;
        let link_os: &OsStr = link.as_ref();
        Ok(Self {
            fd: raw_fd,
            mode: ((md.mode() as RawMode) & Mode::RWXU.bits()) as u16,
            target: expect!(FDTarget::from_str(expect!(link_os.to_str()))),
        })
    }

    /// Gets a file descriptor from a directory fd and a path relative to it.
    ///
    /// `base` is the path to the directory fd, and is used for error messages.
    fn from_process_at<P: AsRef<Path>, Q: AsRef<Path>>(
        base: P,
        dirfd: BorrowedFd,
        path: Q,
        fd: i32,
    ) -> ProcResult<Self> {
        let p = path.as_ref();
        let root = base.as_ref().join(p);
        // for 2.6.39 <= kernel < 3.6 fstat doesn't support O_PATH see https://github.com/eminence/procfs/issues/265
        let flags = match Version::cached() {
            Ok(v) if v < KernelVersion::new(3, 6, 0) => OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Ok(_) => OFlags::NOFOLLOW | OFlags::PATH | OFlags::CLOEXEC,
            Err(_) => OFlags::NOFOLLOW | OFlags::PATH | OFlags::CLOEXEC,
        };
        let file = wrap_io_error!(root, rustix::fs::openat(dirfd, p, flags, Mode::empty()))?;
        let link = rustix::fs::readlinkat(&file, "", Vec::new()).map_err(io::Error::from)?;
        let md =
            rustix::fs::statat(&file, "", AtFlags::SYMLINK_NOFOLLOW | AtFlags::EMPTY_PATH).map_err(io::Error::from)?;

        let link_os = link.to_string_lossy();
        let target = FDTarget::from_str(link_os.as_ref())?;
        Ok(FDInfo {
            fd,
            mode: (md.st_mode & Mode::RWXU.bits()) as u16,
            target,
        })
    }

    /// Gets the read/write mode of this file descriptor as a bitfield
    pub fn mode(&self) -> FDPermissions {
        FDPermissions::from_bits_truncate(self.mode)
    }
}

impl std::fmt::Debug for FDInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FDInfo {{ fd: {:?}, mode: 0{:o}, target: {:?} }}",
            &self.fd, self.mode, self.target
        )
    }
}

/// Represents a process in `/proc/<pid>`.
///
/// **Note** The `Process` struct holds an open file descriptor to its `/proc/<pid>` directory.
/// This makes it possible to construct a `Process` object and then later call the various methods
/// on it without a risk of inadvertently getting information from the wrong process (due to PID
/// reuse).
///
/// However the downside is that holding a lot of `Process` objects might cause the process to run
/// out of file descriptors.
///
/// For use cases that don't involve holding a lot of `Process` objects, no special handler is
/// needed.  But if you do hold a lot of these objects (for example if you're writing a `ps`
/// or `top` -like program), you'll likely want to gather all of the necessary info from `Process`
/// object into a new struct and then drop the `Process` object
///
#[derive(Debug)]
pub struct Process {
    fd: OwnedFd,
    pub pid: i32,
    pub(crate) root: PathBuf,
