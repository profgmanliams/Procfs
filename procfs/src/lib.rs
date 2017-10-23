#![allow(unknown_lints)]
#![deny(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
//! This crate provides to an interface into the linux `procfs` filesystem, usually mounted at
//! `/proc`.
//!
//! This is a pseudo-filesystem which is available on most every linux system and provides an
//! interface to kernel data structures.
//!
//!
//! # Kernel support
//!
//! Not all fields/data are available in each kernel.  Some fields were added in specific kernel
//! releases, and other fields are only present in certain kernel configuration options are
//! enabled.  These are represented as `Option` fields in this crate.
//!
//! This crate aims to support all 2.6 kernels (and newer).  WSL2 is also supported.
//!
//! # Documentation
//!
//! In almost all cases, the documentation is taken from the
//! [`proc.5`](http://man7.org/linux/man-pages/man5/proc.5.html) manual page.  This means that
//! sometimes the style of writing is not very "rusty", or may do things like reference related files
//! (instead of referencing related structs).  Contributions to improve this are welcome.
//!
//! # Panicking
//!
//! While previous versions of the library could panic, this current version aims to be panic-free
//! in a many situations as possible.  Whenever the procfs crate encounters a bug in its own
//! parsing code, it will return an [`InternalError`](enum.ProcError.html#variant.InternalError) error.  This should be considered a
//! bug and should be [reported](https://github.com/eminence/procfs).  If you encounter a panic,
//! please report that as well.
//!
//! # Cargo features
//!
//! The following cargo features are available:
//!
//! * `chrono` -- Default.  Optional.  This feature enables a few methods that return values as `DateTime` objects.
//! * `flate2` -- Default.  Optional.  This feature enables parsing gzip compressed `/proc/config.gz` file via the `procfs::kernel_config` method.
//! * `backtrace` -- Optional.  This feature lets you get a stack trace whenever an `InternalError` is raised.
//!
//! # Examples
//!
//! Examples can be found in the various modules shown below, or in the
//! [examples](https://github.com/eminence/procfs/tree/master/procfs/examples) folder of the code repository.
//!

pub use procfs_core::*;

use bitflags::bitflags;

use rustix::fd::AsFd;
use rustix::fs::{Mode, OFlags};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "chrono")]
use chrono::DateTime;

const PROC_CONFIG_GZ: &str = "/proc/config.gz";
const BOOT_CONFIG: &str = "/boot/config";

/// Allows associating a specific file to parse.
pub trait Current: FromRead {
    const PATH: &'static str;

    /// Parse the current value using the system file.
    fn current() -> ProcResult<Self> {
        Self::from_file(Self::PATH)
    }
}

/// A type for accessing data about the currently running machine
///
/// For more details, see the [SystemInfoInterface] trait.
pub struct LocalSystemInfo;

impl SystemInfoInterface for LocalSystemInfo {
    fn boot_time_secs(&self) -> ProcResult<u64> {
        crate::boot_time_secs()
    }

    fn ticks_per_second(&self) -> u64 {
        crate::ticks_per_second()
    }

    fn page_size(&self) -> u64 {
        crate::page_size()
    }

    fn is_little_endian(&self) -> bool {
        u16::from_ne_bytes([0, 1]).to_le_bytes() == [0, 1]
    }
}

const LOCAL_SYSTEM_INFO: LocalSystemInfo = LocalSystemInfo;

/// The current [SystemInfo].
pub fn current_system_info() -> &'static SystemInfo {
    &LOCAL_SYSTEM_INFO
}

/// Allows associating a specific file to parse with system information.
pub trait CurrentSI: FromReadSI {
    const PATH: &'static str;

    /// Parse the current value using the system file and the current system info.
    fn current() -> ProcResult<Self> {
        Self::current_with_system_info(current_system_info())
    }

    /// Parse the current value using the system file and the provided system info.
    fn current_with_system_info(si: &SystemInfo) -> ProcResult<Self> {
        Self::from_file(Self::PATH, si)
    }
}

/// Allows `impl WithSystemInfo` to use the current system info.
pub trait WithCurrentSystemInfo<'a>: WithSystemInfo<'a> + Sized {
    /// Get the value using the current system info.
    fn get(self) -> Self::Output {
        self.with_system_info(current_system_info())
    }
}

impl<'a, T: WithSystemInfo<'a>> WithCurrentSystemInfo<'a> for T {}

/// Extension traits useful for importing wholesale.
pub mod prelude {
    pub use super::{Current, CurrentSI, WithCurrentSystemInfo};
    pub use procfs_core::prelude::*;
}

macro_rules! wrap_io_error {
    ($path:expr, $expr:expr) => {
        match $expr {
            Ok(v) => Ok(v),
            Err(e) => {
                let kind = e.kind();
                Err(::std::io::Error::new(
                    kind,
                    crate::IoErrorWrapper {
                        path: $path.to_owned(),
                        inner: e.into(),
                    },
                ))
            }
        }
    };
}

pub(crate) fn read_file<P: AsRef<Path>>(path: P) -> ProcResult<String> {
    std::fs::read_to_string(path.as_ref())
        .map_err(|e| e.into())
        .error_path(path.as_ref())
}

pub(crate) fn write_file<P: AsRef<Path>, T: AsRef<[u8]>>(path: P, buf: T) -> ProcResult<()> {
    std::fs::write(path.as_ref(), buf)
        .map_err(|e| e.into())
        .error_path(path.as_ref())
}

pub(crate) fn read_value<P, T, E>(path: P) -> ProcResult<T>
where
    P: AsRef<Path>,
    T: FromStr<Err = E>,
    ProcError: From<E>,
{
    read_file(path).and_then(|s| s.trim().parse().map_err(ProcError::from))
}

pub(crate) fn write_value<P: AsRef<Path>, T: fmt::Display>(path: P, value: T) -> ProcResult<()> {
    write_file(path, value.to_string().as_bytes())
}

mod cgroups;
pub use crate::cgroups::*;

mod crypto;
pub use crate::crypto::*;

pub mod keyring;

mod iomem;
pub use iomem::*;

mod kpageflags;
pub use kpageflags::*;

mod kpagecount;
pub use kpagecount::*;

pub mod net;

pub mod process;

pub mod sys;
pub use crate::sys::kernel::BuildInfo as KernelBuildInfo;
pub use crate::sys::kernel::Type as KernelType;
pub use crate::sys::kernel::Version as KernelVersion;

/// A wrapper around a `File` that remembers the name of the path
struct FileWrapper {
    inner: File,
    path: PathBuf,
}

impl FileWrapper {
    fn open<P: AsRef<Path>>(path: P) -> Result<FileWrapper, io::Error> {
        let p = path.as_ref();
        let f = wrap_io_error!(p, File::open(p))?;
        Ok(FileWrapper {
            inner: f,
            path: p.to_owned(),
        })
    }
    fn open_at_flags<P, Q, Fd: AsFd>(root: P, dirfd: Fd, path: Q, flags: OFlags) -> Result<FileWrapper, io::Error>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let p = root.as_ref().join(path.as_ref());
        let fd = wrap_io_error!(p, rustix::fs::openat(dirfd, path.as_ref(), flags, Mode::empty()))?;
        Ok(FileWrapper {
            inner: File::from(fd),
            path: p,
        })
    }

    fn open_at<P, Q, Fd: AsFd>(root: P, dirfd: Fd, path: Q) -> Result<FileWrapper, io::Error>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        Self::open_at_flags(root, dirfd, path, OFlags::RDONLY | OFlags::CLOEXEC)
    }

    /// Returns the inner file
    fn inner(self) -> File {
        self.inner
    }
}

impl Read for FileWrapper {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        wrap_io_error!(self.path, self.inner.read(buf))
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        wrap_io_error!(self.path, self.inner.read_to_end(buf))
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        wrap_io_error!(self.path, self.inner.read_to_string(buf))
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        wrap_io_error!(self.path, self.inner.read_exact(buf))
    }
}

impl Seek for FileWrapper {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        wrap_io_error!(self.path, self.inner.seek(pos))
    }
}

/// Return the number of ticks per second.
///
/// This isn't part of the proc file system, but it's a useful thing to have, since several fields
/// count in ticks.  This is calculated from `sysconf(_SC_CLK_TCK)`.
pub fn ticks_per_second() -> u64 {
    rustix::param::clock_ticks_per_second()
}

/// The boot time of the system, as a `DateTime` object.
///
/// This is calculated from `/proc/stat`.
///
/// This function requires the "chrono" features to be enabled (which it is by default).
#[cfg(feature = "chrono")]
pub fn boot_time() -> ProcResult<DateTime<chrono::Local>> {
    use chrono::TimeZone;
    let secs = boot_time_secs()?;

    let date_time = expect!(chrono::Local.timestamp_opt(secs as i64, 0).single());

    Ok(date_time)
}

/// The boottime of the system, in seconds since the epoch
///
/// This is calculated from `/proc/stat`.
///
#[cfg_attr(
    not(feature = "chrono"),
    doc = "If you compile with the optional `chrono` feature, you can use the `boot_time()` method to get the boot time as a `DateTime` object."
)]
#[cfg_attr(
    feature = "chrono",
    doc = "See also [boot_time()] to get the boot time as a `DateTime`"
)]
pub fn boot_time_secs() -> ProcResult<u64> {
    BOOT_TIME.with(|x| {
        let mut btime = x.borrow_mut();
        if let Some(btime) = *btime {
            Ok(btime)
        } else {
            // KernelStats doesn't call `boot_time_secs()`, so it's safe to call
            // KernelStats::current() (which uses the local system info).
            let stat = KernelStats::current()?;
            *btime = Some(stat.btime);
            Ok(stat.btime)
        }
    })
}

thread_local! {
    static BOOT_TIME : std::cell::RefCell<Option<u64>> = std::cell::RefCell::new(None);
}

/// Memory page size, in bytes.
///
