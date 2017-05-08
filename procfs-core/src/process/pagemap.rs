use bitflags::bitflags;
use std::{fmt, mem::size_of};

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

const fn genmask(high: usize, low: usize) -> u64 {
    let mask_bits = size_of::<u64>() * 8;
    (!0 - (1 << low) + 1) & (!0 >> (mask_bits - 1 - high))
}

// source: include/linux/swap.h
const MAX_SWAPFILES_SHIFT: usize = 5;

// source: fs/proc/task_mmu.c
bitflags! {
    /// Represents the fields and flags in a page table entry for a swapped page.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct SwapPageFlags: u64 {
        /// Swap type if swapped
        #[doc(hidden)]
        const SWAP_TYPE = genmask(MAX_SWAPFILES_SHIFT - 1, 0);
        /// Swap offset if swapped
        #[doc(hidden)]
        const SWAP_OFFSET = genmask(54, MAX_SWAPFILES_SHIFT);
        /// PTE is soft-dirty
        const SOFT_DIRTY = 1 << 55;
        /// Page is exclusively mapped
        const MMAP_EXCLUSIVE = 1 << 56;
        /// Page is file-page or shared-anon
        const FILE = 1 << 61;
        /// Page is swapped
        #[doc(hidden)]
        const SWAP = 1 << 62;
        /// Page is present
        const PRESENT = 1 << 63;
    }
}

impl SwapPageFlags {
    /// Returns the swap type recorded in this entry.
    pub fn get_swap_type(&self) -> u64 {
        (*self & Self::SWAP_TYPE).bits()
    }

    /// Returns the swap offset recorded in this entry.
    pub fn get_swap_offset(&self) -> u64 {
        (*self & Self::SWAP_OFFSET).bits() >> MAX_SWAPFILES_SHIFT
    }
}

bitflags! {
    /// Represents the fields and flags in a page table entry for a memory page.
    #[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct MemoryPageFlags: u64 {
        /// Page frame number if present
        #[doc(hidden)]
        const PFN = genmask(54, 0);
        /// PTE is soft-dirty
        const SOFT_DIRTY = 1 << 55;
        /// Page is exclusively mapped
        const MMAP_EXCLUSIVE = 1 << 56;
        /// Page is file-page or shared-anon
        const FILE = 1 << 61;
        /// Page is swapped
        #[doc(hidden)]
        const SWAP = 1 << 62;
        /// Page is present
        const PRESENT = 1 << 63;
    }
}

impl MemoryPageFlags {
    /// Returns the page frame number recorded in this entry.
    pub fn get_page_frame_number(&self) -> Pfn {
        Pfn((*self & Self::PFN).bits())
    }
}

/// A Page Frame Number, representing a 4 kiB physical memory page
