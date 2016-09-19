
use procfs_core::ProcResult;
pub use procfs_core::CryptoTable;

use crate::Current;

impl Current for CryptoTable {
    const PATH: &'static str = "/proc/crypto";
}

pub fn crypto() -> ProcResult<CryptoTable> {
    CryptoTable::current()
}
