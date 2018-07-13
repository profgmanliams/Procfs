use bitflags::bitflags;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

//const fn genmask(high: usize, low: usize) -> u64 {
//    let mask_bits = size_of::<u64>() * 8;
//    (!0 - (1 << low) + 1) & (!0 >> (mask_bits - 1 - high))
//}

bitflags! {
    /// Represents the fields and flags in a page table entry for a memory page.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct PhysicalPageFlags: u64 {
        /// The page is being locked for exclusive access, e.g. by undergoing read/write IO
        const LOCKED = 1 << 0;
        /// IO error occurred
        const ERROR = 1 << 1;
        /// The page has been referenced since last LRU list enqueue/requeue
        const REFERENCED = 1 << 2;
        /// The page has up-to-date data. ie. for file backed page: (in-memory data revision >= on-disk one)
