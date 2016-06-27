use super::ProcResult;
use crate::Current;
use procfs_core::{Iomem, PhysicalMemoryMap};

impl Current for Iomem {
    const PATH: &'static str = "/proc/iomem";
}
