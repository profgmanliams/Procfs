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
