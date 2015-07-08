#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};
use std::io::BufRead;

use super::ProcResult;
use crate::{process::Pfn, split_into_num};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct Iomem(pub Vec<(usize, PhysicalMemoryMap)>);

impl crate::FromBufRead for Iomem {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut vec = Vec::new();

        for line in r.lines() {
            let line = expect!(line);

            let (indent, map) = PhysicalMemoryMap::from_line(&line)?;

            vec.push((indent, map));
        }

        Ok(Iomem(vec))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct PhysicalMemoryMap {
    /// The address space in the process that the mapping occupies.
    pub address: (u64, u64),
    pub name: String,
}

impl PhysicalMemoryMap {
    fn from_line(line: &str) -> ProcResult<(usize, PhysicalMemoryMap)> {
        let indent = line.chars().take_while(|c| *c == ' ').count() / 2;
        let line = line.trim();
