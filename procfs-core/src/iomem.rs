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
