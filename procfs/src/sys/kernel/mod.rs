//! Global kernel info / tuning miscellaneous stuff
//!
//! The files in this directory can be used to tune and monitor miscellaneous
//! and general things in the operation of the Linux kernel.

use std::cmp;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, Ordering};

use bitflags::bitflags;

use crate::{read_value, write_value, ProcError, ProcResult};

pub mod keys;
pub mod random;

/// Represents a kernel version, in major.minor.release version.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u16,
}

impl Version {
    pub fn new(major: u8, minor: u8, patch: u16) -> Version {
        Version { major, minor, patch }
    }

    /// Returns the kernel version of the currently running kernel.
    ///
    /// This is taken from `/proc/sys/kernel/osrelease`;
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/osrelease")
    }

    /// Cached version of the kernel version.
    pub(crate) fn cached() -> ProcResult<Self> {
        const SENTINEL: u32 = 0;
        static KERNEL: AtomicU32 = AtomicU32::new(SENTINEL);

        // Try to load the kernel version.
        let mut kernel = KERNEL.load(Ordering::Relaxed);

        // If we haven't loaded the kernel version yet, try to here.
        if kernel == SENTINEL {
            kernel = Self::current()?.to_u32();

            // Try to store it in the cache.
            KERNEL.store(kernel, Ordering::Release);
        }

        Ok(Self::from_u32(kernel))
    }

    /// Convert kernel version to an arbitrary `u32` value.
    fn to_u32(self) -> u32 {
        let [lo, hi] = u16::to_ne_bytes(self.patch);
        u32::from_ne_bytes([self.major, self.minor, lo, hi])
    }

    /// Convert kernel version from an arbitrary `u32` value.
    fn from_u32(val: u32) -> Self {
        let [major, minor, lo, hi] = u32::to_ne_bytes(val);
        Self {
            major,
            minor,
            patch: u16::from_ne_bytes([lo, hi])
        }
    }

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a = KernelVersion::from_str("3.16.0-6-amd64").unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let pos = s.find(|c: char| c != '.' && !c.is_ascii_digit());
        let kernel = if let Some(pos) = pos {
            let (s, _) = s.split_at(pos);
            s
        } else {
            s
        };
        let mut kernel_split = kernel.split('.');

        let major = kernel_split.next().ok_or("Missing major version component")?;
        let minor = kernel_split.next().ok_or("Missing minor version component")?;
        let patch = kernel_split.next().ok_or("Missing patch version component")?;

        let major = major.parse().map_err(|_| "Failed to parse major version")?;
        let minor = minor.parse().map_err(|_| "Failed to parse minor version")?;
        let patch = patch.parse().map_err(|_| "Failed to parse patch version")?;

        Ok(Version { major, minor, patch })
    }
}

impl FromStr for Version {
    type Err = &'static str;

    /// Parses a kernel version string, in major.minor.release syntax.
    ///
    /// Note that any extra information (stuff after a dash) is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// # use procfs::KernelVersion;
    /// let a: KernelVersion = "3.16.0-6-amd64".parse().unwrap();
    /// let b = KernelVersion::new(3, 16, 0);
    /// assert_eq!(a, b);
    ///
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::from_str(s)
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.major.cmp(&other.major) {
            cmp::Ordering::Equal => match self.minor.cmp(&other.minor) {
                cmp::Ordering::Equal => self.patch.cmp(&other.patch),
                x => x,
            },
            x => x,
        }
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a kernel type
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub sysname: String,
}

impl Type {
    pub fn new(sysname: String) -> Type {
        Type { sysname }
    }

    /// Read the kernel type from current running kernel
    ///
    /// Defined in `include/linux/uts.h` as UTS_SYSNAME, default is "Linux".
    /// The file is located at `/proc/sys/kernel/ostype`.
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/ostype")
    }
}

impl FromStr for Type {
    type Err = &'static str;

    /// Parse a kernel type string
    ///
    /// Notice that in Linux source code, it is defined as a single string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Type::new(s.to_string()))
    }
}

/// Represents a kernel build information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuildInfo {
    pub version: String,
    pub flags: HashSet<String>,
    /// This field contains any extra data from the /proc/sys/kernel/version file. It generally contains the build date of the kernel, but the format of the date can vary.
    ///
    /// A method named `extra_date` is provided which would try to parse some date formats. When the date format is not supported, an error will be returned. It depends on chrono feature.
    pub extra: String,
}

impl BuildInfo {
    pub fn new(version: &str, flags: HashSet<String>, extra: String) -> BuildInfo {
        BuildInfo {
            version: version.to_string(),
            flags,
            extra,
        }
    }

    /// Read the kernel build information from current running kernel
    ///
    /// Generated by `scripts/mkcompile_h` when building the kernel.
    /// The file is located at `/proc/sys/kernel/version`.
    pub fn current() -> ProcResult<Self> {
        read_value("/proc/sys/kernel/version")
    }

    /// Check if SMP is ON
    pub fn smp(&self) -> bool {
