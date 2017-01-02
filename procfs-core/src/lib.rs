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
