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
}

/// Methods for constructing a new `Process` object.
impl Process {
    /// Returns a `Process` based on a specified PID.
    ///
    /// This can fail if the process doesn't exist, or if you don't have permission to access it.
    pub fn new(pid: i32) -> ProcResult<Process> {
        let root = PathBuf::from("/proc").join(pid.to_string());
        Self::new_with_root(root)
    }

    /// Returns a `Process` based on a specified `/proc/<pid>` path.
    pub fn new_with_root(root: PathBuf) -> ProcResult<Process> {
        // for 2.6.39 <= kernel < 3.6 fstat doesn't support O_PATH see https://github.com/eminence/procfs/issues/265
        let flags = match Version::cached() {
            Ok(v) if v < KernelVersion::new(3, 6, 0) => OFlags::DIRECTORY | OFlags::CLOEXEC,
            Ok(_) => OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Err(_) => OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
        };
        let file = wrap_io_error!(root, rustix::fs::openat(rustix::fs::CWD, &root, flags, Mode::empty()))?;

        let pidres = root
            .as_path()
            .components()
            .last()
            .and_then(|c| match c {
                std::path::Component::Normal(s) => Some(s),
                _ => None,
            })
            .and_then(|s| s.to_string_lossy().parse::<i32>().ok())
            .or_else(|| {
                rustix::fs::readlinkat(rustix::fs::CWD, &root, Vec::new())
                    .ok()
                    .and_then(|s| s.to_string_lossy().parse::<i32>().ok())
            });
        let pid = match pidres {
            Some(pid) => pid,
            None => return Err(ProcError::NotFound(Some(root))),
        };

        Ok(Process { fd: file, pid, root })
    }

    /// Returns a `Process` for the currently running process.
    ///
    /// This is done by using the `/proc/self` symlink
    pub fn myself() -> ProcResult<Process> {
        let root = PathBuf::from("/proc/self");
        Self::new_with_root(root)
    }
}

impl Process {
    /// Returns the complete command line for the process, unless the process is a zombie.
    pub fn cmdline(&self) -> ProcResult<Vec<String>> {
        let mut buf = String::new();
        let mut f = FileWrapper::open_at(&self.root, &self.fd, "cmdline")?;
        f.read_to_string(&mut buf)?;
        Ok(buf
            .split('\0')
            .filter_map(|s| if !s.is_empty() { Some(s.to_string()) } else { None })
            .collect())
    }

    /// Returns the process ID for this process, if the process was created from an ID. Otherwise
    /// use stat().pid.
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// Is this process still alive?
    ///
    /// Processes in the Zombie or Dead state are not considered alive.
    pub fn is_alive(&self) -> bool {
        if let Ok(stat) = self.stat() {
            stat.state != 'Z' && stat.state != 'X'
        } else {
            false
        }
    }

    /// What group owns this process?
    pub fn gid(&self) -> ProcResult<u32> {
        Ok(self.metadata()?.st_gid)
    }

    /// What user owns this process?
    pub fn uid(&self) -> ProcResult<u32> {
        Ok(self.metadata()?.st_uid)
    }

    fn metadata(&self) -> ProcResult<rustix::fs::Stat> {
        Ok(rustix::fs::fstat(&self.fd).map_err(io::Error::from)?)
    }

    /// Retrieves current working directory of the process by dereferencing `/proc/<pid>/cwd` symbolic link.
    ///
    /// This method has the following caveats:
    ///
    /// * if the pathname has been unlinked, the symbolic link will contain the string " (deleted)"
    ///   appended to the original pathname
    ///
    /// * in a multithreaded process, the contents of this symbolic link are not available if the
    ///   main thread has already terminated (typically by calling `pthread_exit(3)`)
    ///
    /// * permission to dereference or read this symbolic link is governed by a
    ///   `ptrace(2)` access mode `PTRACE_MODE_READ_FSCREDS` check
    pub fn cwd(&self) -> ProcResult<PathBuf> {
        Ok(PathBuf::from(OsString::from_vec(
            wrap_io_error!(
                self.root.join("cwd"),
                rustix::fs::readlinkat(&self.fd, "cwd", Vec::new())
            )?
            .into_bytes(),
        )))
    }

    /// Retrieves current root directory of the process by dereferencing `/proc/<pid>/root` symbolic link.
    ///
    /// This method has the following caveats:
    ///
    /// * if the pathname has been unlinked, the symbolic link will contain the string " (deleted)"
    ///   appended to the original pathname
    ///
    /// * in a multithreaded process, the contents of this symbolic link are not available if the
    ///   main thread has already terminated (typically by calling `pthread_exit(3)`)
    ///
    /// * permission to dereference or read this symbolic link is governed by a
    ///   `ptrace(2)` access mode `PTRACE_MODE_READ_FSCREDS` check
    pub fn root(&self) -> ProcResult<PathBuf> {
        Ok(PathBuf::from(OsString::from_vec(
            wrap_io_error!(
                self.root.join("root"),
                rustix::fs::readlinkat(&self.fd, "root", Vec::new())
            )?
            .into_bytes(),
        )))
    }

    /// Gets the current environment for the process.  This is done by reading the
    /// `/proc/pid/environ` file.
    pub fn environ(&self) -> ProcResult<HashMap<OsString, OsString>> {
        use std::os::unix::ffi::OsStrExt;

        let mut map = HashMap::new();

        let mut file = FileWrapper::open_at(&self.root, &self.fd, "environ")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        for slice in buf.split(|b| *b == 0) {
            // slice will be in the form key=var, so split on the first equals sign
            let mut split = slice.splitn(2, |b| *b == b'=');
            if let (Some(k), Some(v)) = (split.next(), split.next()) {
                map.insert(OsStr::from_bytes(k).to_os_string(), OsStr::from_bytes(v).to_os_string());
            };
            //let env = OsStr::from_bytes(slice);
        }

        Ok(map)
    }

    /// Retrieves the actual path of the executed command by dereferencing `/proc/<pid>/exe` symbolic link.
    ///
    /// This method has the following caveats:
    ///
    /// * if the pathname has been unlinked, the symbolic link will contain the string " (deleted)"
    ///   appended to the original pathname
    ///
    /// * in a multithreaded process, the contents of this symbolic link are not available if the
    ///   main thread has already terminated (typically by calling `pthread_exit(3)`)
    ///
    /// * permission to dereference or read this symbolic link is governed by a
    ///   `ptrace(2)` access mode `PTRACE_MODE_READ_FSCREDS` check
    pub fn exe(&self) -> ProcResult<PathBuf> {
        Ok(PathBuf::from(OsString::from_vec(
            wrap_io_error!(
                self.root.join("exe"),
                rustix::fs::readlinkat(&self.fd, "exe", Vec::new())
            )?
            .into_bytes(),
        )))
    }

    /// Return the Io stats for this process, based on the `/proc/pid/io` file.
    ///
    /// (since kernel 2.6.20)
    pub fn io(&self) -> ProcResult<Io> {
        FromRead::from_read(FileWrapper::open_at(&self.root, &self.fd, "io")?)
    }

    /// Return a list of the currently mapped memory regions and their access permissions, based on
    /// the `/proc/pid/maps` file.
    pub fn maps(&self) -> ProcResult<MemoryMaps> {
        FromRead::from_read(FileWrapper::open_at(&self.root, &self.fd, "maps")?)
    }

    /// Returns a list of currently mapped memory regions and verbose information about them,
    /// such as memory consumption per mapping, based on the `/proc/pid/smaps` file.
    ///
    /// (since Linux 2.6.14 and requires CONFIG_PROG_PAGE_MONITOR)
    pub fn smaps(&self) -> ProcResult<MemoryMaps> {
        FromRead::from_read(FileWrapper::open_at(&self.root, &self.fd, "smaps")?)
    }

    /// This is the sum of all the smaps data but it is much more performant to get it this way.
    ///
    /// Since 4.14 and requires CONFIG_PROC_PAGE_MONITOR.
    pub fn smaps_rollup(&self) -> ProcResult<SmapsRollup> {
        FromRead::from_read(FileWrapper::open_at(&self.root, &self.fd, "smaps_rollup")?)
    }
