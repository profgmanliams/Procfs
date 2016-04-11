use crate::{FileWrapper, ProcResult};
use procfs_core::process::PageInfo;
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    mem::size_of,
    ops::{Bound, RangeBounds},
};

impl super::Process {
    /// Returns a struct that can be used to access information in the `/proc/pid/pagemap` file.
    pub fn pagemap(&self) -> ProcResult<PageMap> {
        let path = self.root.join("pagemap");
        let file = FileWrapper::open(&path)?;
        Ok(PageMap::from_file_wrapper(file))
    }
}

/// Parses page table entries accessing `/proc/<pid>/pagemap`.
pub struct PageMap {
    reader: BufReader<FileWrapper>,
}

impl PageMap {
    pub(crate) fn from_file_wrapper(file: FileWrapper) -> Self {
        Self {
            reader: BufReader::new(file),
        }
    }

    /// Retrieves information in the page table entry for the page at index `page_index`.
    ///
    /// Some mappings are not accessible, and will return an Err: `vsyscall`
    pub fn get_info(&mut self, page_index: usize) -> ProcResult<PageInfo> {
