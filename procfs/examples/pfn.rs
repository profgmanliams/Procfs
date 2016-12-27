//
// Print physical memory location for each page for each memory mapping
// This requires CAP_SYS_ADMIN privilege, or root, otherwise physical memory addresses will be zero
//
// Vocabulary
// VA = Virtual Address: memory address from a process point of view
// VPN = Virtual Page Number: page number of the a Virtual Memory address
// PA = Physical Address: memory address in physical memory
// PFN = Page Frame Number: page number of a Physical Address
//

use procfs::process::MMapPath;
use procfs::process::Process;

fn main() {
    if !rustix::process::geteuid().is_root() {
        println!("WARNING: Access to /proc/<PID>/pagemap requires root, re-run with sudo");
    }

    let page_size = procfs::page_size();

    let process = Process::myself().expect("Unable to load myself!");

    let mut pagemap = process.pagemap().unwrap();
    let mem_map = process.maps().unwrap();

