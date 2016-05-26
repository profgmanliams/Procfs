use crate::from_iter;
use crate::ProcResult;
use std::io::Read;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Provides scheduler statistics of the process, based on the `/proc/<pid>/schedstat` file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
