use crate::ProcResult;
#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// Container group controller information.
pub struct CGroupController {
    /// The name of the controller.
    pub name: String,
    /// The  unique  ID  of  the  cgroup hierarchy on which this controller is mounted.
    ///
    /// If multiple cgroups v1 controllers are bound to the same  hierarchy, then each will show
    /// the same hierarchy ID in this field.  The value in this field will be 0 if:
    ///
    /// * the controller is not mounted on a cgroups v1 hierarchy;
    /// * the controller is bound to the cgroups v2 single unified hierarchy; or
    /// * the controller is disabled (see below).
    pub hierarchy: u32,
    /// The number of control groups in this hierarchy using this controller.
    pub num_cgroups: u32,
    /// This field contains the value `true` if this controller is enabled, or `false` if it has been disabled
    pub enabled: bool,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// Container group controller information.
// This contains a vector, but if each subsystem name is unique, maybe this can be a
// hashmap instead
pub struct CGroupControllers(pub Vec<CGroupController>);

