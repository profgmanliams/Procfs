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
        const UPTODATE = 1 << 3;
        /// The page has been written to, hence contains new data. i.e. for file backed page: (in-memory data revision > on-disk one)
        const DIRTY = 1 << 4;
        /// The page is in one of the LRU lists
        const LRU = 1 << 5;
        /// The page is in the active LRU list
        const ACTIVE = 1 << 6;
        /// The page is managed by the SLAB/SLOB/SLUB/SLQB kernel memory allocator. When compound page is used, SLUB/SLQB will only set this flag on the head page; SLOB will not flag it at all
        const SLAB = 1 << 7;
        /// The page is being synced to disk
        const WRITEBACK = 1 << 8;
        /// The page will be reclaimed soon after its pageout IO completed
        const RECLAIM = 1 << 9;
        /// A free memory block managed by the buddy system allocator. The buddy system organizes free memory in blocks of various orders. An order N block has 2^N physically contiguous pages, with the BUDDY flag set for and _only_ for the first page
        const BUDDY = 1 << 10;
        /// A memory mapped page
        const MMAP = 1 << 11;
        /// A memory mapped page that is not part of a file
        const ANON = 1 << 12;
        /// The page is mapped to swap space, i.e. has an associated swap entry
        const SWAPCACHE = 1 << 13;
        /// The page is backed by swap/RAM
        const SWAPBACKED = 1 << 14;
