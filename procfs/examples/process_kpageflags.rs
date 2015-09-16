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
        // KpageFlags::new().unwrap() will panic either way
        panic!("ERROR: Access to /proc/kpageflags requires root, re-run with sudo");
    }

    let page_size = procfs::page_size();

    // We will inspect this process's own memory
    let process = Process::myself().expect("Unable to load myself!");
    let mut kpageflags = KPageFlags::new().expect("Can't open /proc/kpageflags");

    let mut pagemap = process.pagemap().unwrap();

    // The memory maps are read now, so the value we look for must already exist in RAM when we it this line
    // In this case it works, because the variables already exist in the executable
    // You probably want to put this right above the "for memory_map" loop
    let mem_map = process.maps().unwrap();

    // We allocate memory for a value. This is a trick to get a semi random value
    // The goal is to find this value in physical memory
    let chrono = std::time::Instant::now();
