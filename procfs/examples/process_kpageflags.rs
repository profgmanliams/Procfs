//
// Look for a value in the virtual memory of a process, and physical memory, then prints memory page details
// This shows how to go from virtual address to mapping, and from mapping to physical address.
//
// This requires CAP_SYS_ADMIN privilege, or root
//
// Sample output:
//
// Virtual address of `variable`: 0x7ffd2de4708f
// Found memory mapping
// MemoryMap { address: (140725373272064, 140725373407232), perms: "rw-p", offset: 0, dev: (0, 0), inode: 0, pathname: Stack }
// Found page
// virt_mem: 0x7ffd2de47000, pfn: 0x107b06, phys_addr: 0x107b06000, flags: UPTODATE | LRU | MMAP | ANON | SWAPBACKED
//

use procfs::process::Process;
use procfs::KPageFlags;

fn main() {
    if !rustix::process::geteuid().is_root() {
