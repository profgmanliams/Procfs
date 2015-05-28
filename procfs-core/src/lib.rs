#![allow(unknown_lints)]
#![deny(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
//! This crate provides to an interface into the linux `procfs` filesystem, usually mounted at
//! `/proc`.
//!
//! This is a pseudo-filesystem which is available on most every linux system and provides an
//! interface to kernel data structures.
//!
//! # `procfs-core`
//!
//! The `procfs-core` crate is a fully platform-independent crate that contains most of the data-structures and
//! parsing code.  Most people should first look at the `procfs` crate instead.

use bitflags::bitflags;

use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{collections::HashMap, time::Duration};

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Types which can be parsed from a Read implementation.
pub trait FromRead: Sized {
    /// Read the type from a Read.
    fn from_read<R: Read>(r: R) -> ProcResult<Self>;

    /// Read the type from a file.
    fn from_file<P: AsRef<Path>>(path: P) -> ProcResult<Self> {
        std::fs::File::open(path.as_ref())
            .map_err(|e| e.into())
            .and_then(|f| Self::from_read(f))
            .map_err(|e| e.error_path(path.as_ref()))
    }
}

/// Types which can be parsed from a BufRead implementation.
pub trait FromBufRead: Sized {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self>;
}

impl<T: FromBufRead> FromRead for T {
    fn from_read<R: Read>(r: R) -> ProcResult<Self> {
        Self::from_buf_read(BufReader::new(r))
    }
}

/// Types which can be parsed from a Read implementation and system info.
pub trait FromReadSI: Sized {
    /// Parse the type from a Read and system info.
    fn from_read<R: Read>(r: R, system_info: &SystemInfo) -> ProcResult<Self>;

    /// Parse the type from a file.
    fn from_file<P: AsRef<Path>>(path: P, system_info: &SystemInfo) -> ProcResult<Self> {
        std::fs::File::open(path.as_ref())
            .map_err(|e| e.into())
            .and_then(|f| Self::from_read(f, system_info))
            .map_err(|e| e.error_path(path.as_ref()))
    }
}

/// Types which can be parsed from a BufRead implementation and system info.
pub trait FromBufReadSI: Sized {
    fn from_buf_read<R: BufRead>(r: R, system_info: &SystemInfo) -> ProcResult<Self>;
}

impl<T: FromBufReadSI> FromReadSI for T {
    fn from_read<R: Read>(r: R, system_info: &SystemInfo) -> ProcResult<Self> {
        Self::from_buf_read(BufReader::new(r), system_info)
    }
}

/// Extension traits useful for importing wholesale.
pub mod prelude {
    pub use super::{FromBufRead, FromBufReadSI, FromRead, FromReadSI};
}

#[doc(hidden)]
pub trait IntoOption<T> {
    fn into_option(t: Self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(t: Option<T>) -> Option<T> {
        t
    }
}

impl<T, R> IntoOption<T> for Result<T, R> {
    fn into_option(t: Result<T, R>) -> Option<T> {
        t.ok()
    }
}

#[doc(hidden)]
pub trait IntoResult<T, E> {
    fn into(t: Self) -> Result<T, E>;
}

#[macro_export]
#[doc(hidden)]
macro_rules! build_internal_error {
    ($err: expr) => {
        $crate::ProcError::InternalError($crate::InternalError {
            msg: format!("Internal Unwrap Error: {}", $err),
            file: file!(),
            line: line!(),
            #[cfg(feature = "backtrace")]
            backtrace: backtrace::Backtrace::new(),
        })
    };
    ($err: expr, $msg: expr) => {
        $crate::ProcError::InternalError($crate::InternalError {
            msg: format!("Internal Unwrap Error: {}: {}", $msg, $err),
            file: file!(),
            line: line!(),
            #[cfg(feature = "backtrace")]
            backtrace: backtrace::Backtrace::new(),
        })
    };
}

// custom NoneError, since std::option::NoneError is nightly-only
// See https://github.com/rust-lang/rust/issues/42327
#[doc(hidden)]
pub struct NoneError;

impl std::fmt::Display for NoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoneError")
    }
}

impl<T> IntoResult<T, NoneError> for Option<T> {
    fn into(t: Option<T>) -> Result<T, NoneError> {
        t.ok_or(NoneError)
    }
}

impl<T, E> IntoResult<T, E> for Result<T, E> {
    fn into(t: Result<T, E>) -> Result<T, E> {
        t
    }
}

#[allow(unused_macros)]
#[macro_export]
#[doc(hidden)]
macro_rules! proc_panic {
    ($e:expr) => {
        $crate::IntoOption::into_option($e).unwrap_or_else(|| {
            panic!(
                "Failed to unwrap {}. Please report this as a procfs bug.",
                stringify!($e)
            )
        })
    };
    ($e:expr, $msg:expr) => {
        $crate::IntoOption::into_option($e).unwrap_or_else(|| {
            panic!(
                "Failed to unwrap {} ({}). Please report this as a procfs bug.",
                stringify!($e),
                $msg
            )
        })
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! expect {
    ($e:expr) => {
        match $crate::IntoResult::into($e) {
            Ok(v) => v,
            Err(e) => return Err($crate::build_internal_error!(e)),
        }
    };
    ($e:expr, $msg:expr) => {
        match $crate::IntoResult::into($e) {
            Ok(v) => v,
            Err(e) => return Err($crate::build_internal_error!(e, $msg)),
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! from_str {
    ($t:tt, $e:expr) => {{
        let e = $e;
        $crate::expect!(
            $t::from_str_radix(e, 10),
            format!("Failed to parse {} ({:?}) as a {}", stringify!($e), e, stringify!($t),)
        )
    }};
    ($t:tt, $e:expr, $radix:expr) => {{
        let e = $e;
        $crate::expect!(
            $t::from_str_radix(e, $radix),
            format!("Failed to parse {} ({:?}) as a {}", stringify!($e), e, stringify!($t))
        )
    }};
    ($t:tt, $e:expr, $radix:expr, pid:$pid:expr) => {{
        let e = $e;
        $crate::expect!(
            $t::from_str_radix(e, $radix),
            format!(
                "Failed to parse {} ({:?}) as a {} (pid {})",
                stringify!($e),
                e,
                stringify!($t),
                $pid
            )
        )
    }};
}

/// Auxiliary system information interface.
///
/// A few function in this crate require some extra system info to compute their results.  For example,
/// the [crate::process::Stat::rss_bytes()] function needs to know the page size.  Since `procfs-core` only parses
/// data and never interacts with a real system, this `SystemInfoInterface` is what allows real system info to be used.
///
/// If you are a user of the `procfs` crate, you'll normally use the `[procfs::WithCurrentSystemInfo]` trait.
/// For example:
///
/// ```rust,ignore
/// use procfs::WithCurrentSystemInfo;
///
/// let me = procfs::process::Process::myself().unwrap();
/// let stat = me.stat().unwrap();
/// let bytes = stat.rss_bytes().get();
/// ```
///
/// However, imagine that you captured a process's stat info, along with page size:
/// ```rust
/// # use procfs_core::{FromRead, WithSystemInfo};
/// # let stat_data = std::io::Cursor::new(b"475071 (cat) R 323893 475071 323893 34826 475071 4194304 94 0 0 0 0 0 0 0 20 0 1 0 201288208 5738496 225 18446744073709551615 94881179934720 94881179954601 140722831478832 0 0 0 0 0 0 0 0 0 17 4 0 0 0 0 0 94881179970608 94881179972224 94881184485376 140722831483757 140722831483777 140722831483777 140722831486955 0");
/// let stat = procfs_core::process::Stat::from_read(stat_data).unwrap();
///
/// let system_info = procfs_core::ExplicitSystemInfo {
///     boot_time_secs: 1692972606,
///     ticks_per_second: 100,
///     page_size: 4096,
///     is_little_endian: true,
/// };
///
/// let rss_bytes = stat.rss_bytes().with_system_info(&system_info);
/// ```
pub trait SystemInfoInterface {
    fn boot_time_secs(&self) -> ProcResult<u64>;
    fn ticks_per_second(&self) -> u64;
    fn page_size(&self) -> u64;
    /// Whether the system is little endian (true) or big endian (false).
    fn is_little_endian(&self) -> bool;

    #[cfg(feature = "chrono")]
    fn boot_time(&self) -> ProcResult<chrono::DateTime<chrono::Local>> {
        use chrono::TimeZone;
        let date_time = expect!(chrono::Local.timestamp_opt(self.boot_time_secs()? as i64, 0).single());
        Ok(date_time)
    }
}

/// Auxiliary system information.
pub type SystemInfo = dyn SystemInfoInterface;

/// A convenience struct implementing [SystemInfoInterface] with explicitly-specified values.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct ExplicitSystemInfo {
    pub boot_time_secs: u64,
    pub ticks_per_second: u64,
    pub page_size: u64,
    pub is_little_endian: bool,
}

impl SystemInfoInterface for ExplicitSystemInfo {
    fn boot_time_secs(&self) -> ProcResult<u64> {
        Ok(self.boot_time_secs)
    }

    fn ticks_per_second(&self) -> u64 {
        self.ticks_per_second
    }

    fn page_size(&self) -> u64 {
        self.page_size
    }

    fn is_little_endian(&self) -> bool {
        self.is_little_endian
    }
}

/// Values which can provide an output given the [SystemInfo].
pub trait WithSystemInfo<'a>: 'a {
    type Output: 'a;

    /// Get the output derived from the given [SystemInfo].
    fn with_system_info(self, info: &SystemInfo) -> Self::Output;
}

impl<'a, F: 'a, R: 'a> WithSystemInfo<'a> for F
where
    F: FnOnce(&SystemInfo) -> R,
{
    type Output = R;

    fn with_system_info(self, info: &SystemInfo) -> Self::Output {
        self(info)
    }
}

#[doc(hidden)]
pub fn from_iter<'a, I, U>(i: I) -> ProcResult<U>
where
    I: IntoIterator<Item = &'a str>,
    U: FromStr,
{
    let mut iter = i.into_iter();
    let val = expect!(iter.next());
    match FromStr::from_str(val) {
        Ok(u) => Ok(u),
        Err(..) => Err(build_internal_error!("Failed to convert", val)),
    }
}

fn from_iter_optional<'a, I, U>(i: I) -> ProcResult<Option<U>>
where
    I: IntoIterator<Item = &'a str>,
    U: FromStr,
{
    let mut iter = i.into_iter();
    let Some(val) = iter.next() else {
        return Ok(None);
    };
    match FromStr::from_str(val) {
        Ok(u) => Ok(Some(u)),
        Err(..) => Err(build_internal_error!("Failed to convert (optional)", val)),
    }
}

fn from_iter_radix<'a, I, U>(i: I, radix: u32) -> ProcResult<U>
where
    I: IntoIterator<Item = &'a str>,
    U: FromStrRadix,
{
    let mut iter = i.into_iter();
    let val = expect!(iter.next());

    let val = match radix {
        16 => {
            if let Some(val) = val.strip_prefix("0x") {
                val
            } else if let Some(val) = val.strip_prefix("0X") {
                val
            } else {
                val
            }
        }
        _ => val,
    };

    match FromStrRadix::from_str_radix(val, radix) {
        Ok(u) => Ok(u),
        Err(..) => Err(build_internal_error!("Failed to convert (radix)", val)),
    }
}

mod cgroups;
pub use cgroups::*;

mod cpuinfo;
pub use cpuinfo::*;

mod crypto;
pub use crypto::*;

mod devices;
pub use devices::*;

mod diskstats;
pub use diskstats::*;

mod iomem;
pub use iomem::*;

pub mod keyring;

mod locks;
pub use locks::*;

mod mounts;
pub use mounts::*;

mod partitions;
pub use partitions::*;

mod meminfo;
pub use meminfo::*;

pub mod net;

mod pressure;
pub use pressure::*;

pub mod process;

mod kpageflags;
pub use kpageflags::*;

pub mod sys;
pub use sys::kernel::Version as KernelVersion;

mod sysvipc_shm;
pub use sysvipc_shm::*;

mod uptime;
pub use uptime::*;

// TODO temporary, only for procfs
pub trait FromStrRadix: Sized {
    fn from_str_radix(t: &str, radix: u32) -> Result<Self, std::num::ParseIntError>;
}

impl FromStrRadix for u64 {
    fn from_str_radix(s: &str, radix: u32) -> Result<u64, std::num::ParseIntError> {
        u64::from_str_radix(s, radix)
    }
}

impl FromStrRadix for u32 {
    fn from_str_radix(s: &str, radix: u32) -> Result<u32, std::num::ParseIntError> {
        u32::from_str_radix(s, radix)
    }
}

impl FromStrRadix for i32 {
    fn from_str_radix(s: &str, radix: u32) -> Result<i32, std::num::ParseIntError> {
        i32::from_str_radix(s, radix)
    }
}

fn split_into_num<T: FromStrRadix>(s: &str, sep: char, radix: u32) -> ProcResult<(T, T)> {
    let mut s = s.split(sep);
    let a = expect!(FromStrRadix::from_str_radix(expect!(s.next()), radix));
    let b = expect!(FromStrRadix::from_str_radix(expect!(s.next()), radix));
    Ok((a, b))
}

/// This is used to hold both an IO error as well as the path of the file that originated the error
#[derive(Debug)]
#[doc(hidden)]
pub struct IoErrorWrapper {
    pub path: PathBuf,
    pub inner: std::io::Error,
}

impl std::error::Error for IoErrorWrapper {}
impl fmt::Display for IoErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "IoErrorWrapper({}): {}", self.path.display(), self.inner)
    }
}

/// The main error type for the procfs crate.
///
/// For more info, see the [ProcError] type.
pub type ProcResult<T> = Result<T, ProcError>;

/// The various error conditions in the procfs crate.
///
/// Most of the variants have an `Option<PathBuf>` component.  If the error root cause was related
/// to some operation on a file, the path of this file will be stored in this component.
#[derive(Debug)]
pub enum ProcError {
    /// A standard permission denied error.
    ///
    /// This will be a common error, since some files in the procfs filesystem are only readable by
    /// the root user.
    PermissionDenied(Option<PathBuf>),
    /// This might mean that the process no longer exists, or that your kernel doesn't support the
    /// feature you are trying to use.
    NotFound(Option<PathBuf>),
    /// This might mean that a procfs file has incomplete contents.
    ///
    /// If you encounter this error, consider retrying the operation.
    Incomplete(Option<PathBuf>),
    /// Any other IO error (rare).
    Io(std::io::Error, Option<PathBuf>),
    /// Any other non-IO error (very rare).
    Other(String),
    /// This error indicates that some unexpected error occurred.  This is a bug.  The inner
    /// [InternalError] struct will contain some more info.
    ///
    /// If you ever encounter this error, consider it a bug in the procfs crate and please report
    /// it on github.
    InternalError(InternalError),
}

/// Extensions for dealing with ProcErrors.
pub trait ProcErrorExt {
    /// Add path information to the error.
    fn error_path(self, path: &Path) -> Self;
}

impl ProcErrorExt for ProcError {
    fn error_path(mut self, path: &Path) -> Self {
        use ProcError::*;
        match &mut self {
            PermissionDenied(p) | NotFound(p) | Incomplete(p) | Io(_, p) if p.is_none() => {
                *p = Some(path.to_owned());
            }
            _ => (),
        }
        self
    }
}

impl<T> ProcErrorExt for ProcResult<T> {
    fn error_path(self, path: &Path) -> Self {
        self.map_err(|e| e.error_path(path))
    }
}

/// An internal error in the procfs crate
///
/// If you encounter this error, consider it a bug and please report it on
/// [github](https://github.com/eminence/procfs).
///
/// If you compile with the optional `backtrace` feature (disabled by default),
/// you can gain access to a stack trace of where the error happened.
#[cfg_attr(feature = "serde1", derive(Serialize))]
pub struct InternalError {
    pub msg: String,
    pub file: &'static str,
    pub line: u32,
    #[cfg(feature = "backtrace")]
    #[cfg_attr(feature = "serde1", serde(skip))]
    pub backtrace: backtrace::Backtrace,
}

impl std::fmt::Debug for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bug at {}:{} (please report this procfs bug)\n{}",
            self.file, self.line, self.msg
        )
    }
}

impl std::fmt::Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bug at {}:{} (please report this procfs bug)\n{}",
            self.file, self.line, self.msg
        )
    }
}

impl From<std::io::Error> for ProcError {
    fn from(io: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let kind = io.kind();
        // the only way we'll have a path for the IO error is if this IO error
        // has a inner type
        if io.get_ref().is_some() {
            let inner = io.into_inner().unwrap();

            // is this inner type a IoErrorWrapper?
            match inner.downcast::<IoErrorWrapper>() {
                Ok(wrapper) => {
                    let path = wrapper.path;
                    match kind {
                        ErrorKind::PermissionDenied => ProcError::PermissionDenied(Some(path)),
                        ErrorKind::NotFound => ProcError::NotFound(Some(path)),
                        _other => {
                            // All platforms happen to have ESRCH=3, and windows actually
                            // translates it to a `NotFound` anyway.
                            const ESRCH: i32 = 3;
                            if matches!(wrapper.inner.raw_os_error(), Some(raw) if raw == ESRCH) {
                                // This "No such process" error gets mapped into a NotFound error
                                return ProcError::NotFound(Some(path));
                            } else {
                                ProcError::Io(wrapper.inner, Some(path))
                            }
                        }
                    }
                }
                Err(io) => {
                    // reconstruct the original error
                    ProcError::Io(std::io::Error::new(kind, io), None)
                }
            }
        } else {
            match kind {
                ErrorKind::PermissionDenied => ProcError::PermissionDenied(None),
                ErrorKind::NotFound => ProcError::NotFound(None),
                _other => ProcError::Io(io, None),
            }
        }
    }
}

impl From<&'static str> for ProcError {
    fn from(val: &'static str) -> Self {
        ProcError::Other(val.to_owned())
    }
}

impl From<std::num::ParseIntError> for ProcError {
