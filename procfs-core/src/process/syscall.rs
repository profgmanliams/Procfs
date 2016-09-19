use crate::{from_iter, from_iter_radix, ProcResult};
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

use std::io::Read;

/// Syscall information about the process, based on the `/proc/<pid>/syscall` file.
///
/// New variants to this enum may be added at any time (even without a major or minor semver bump).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
#[non_exhaustive]
