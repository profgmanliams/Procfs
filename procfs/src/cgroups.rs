use super::process::Process;
use crate::{Current, ProcResult};
use procfs_core::CGroupControllers;
pub use procfs_core::ProcessCGroups;

impl Current for CGroupControllers {
    const PATH: &'static str = "/proc/cgroups";
}

/// Information about the cgroup controllers that are compiled into the kernel
///
/// (since Linux 2.6.24)
pub fn cgroups() -> ProcResult<CGroupControllers> {
    CGroupControllers::current()
