use super::MemoryMaps;
use crate::ProcResult;

#[derive(Debug)]
pub struct SmapsRollup {
    pub memory_map_rollup: MemoryMaps,
}

impl crate::FromBufRead for SmapsRollup {
