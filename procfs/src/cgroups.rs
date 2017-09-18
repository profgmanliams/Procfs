use super::process::Process;
use crate::{Current, ProcResult};
use procfs_core::CGroupControllers;
pub use procfs_core::ProcessCGroups;

impl Current for CGroupControllers {
    const PATH: &'static str = "/proc/cgroups";
}
