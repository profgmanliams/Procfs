//
// Print physical location of system RAM
// This requires CAP_SYS_ADMIN privilege, or root, otherwise physical memory addresses will be zero
//

fn main() {
    if !rustix::process::geteuid().is_root() {
        println!("WARNING: Access to /proc/iomem requires root, re-run with sudo");
    }

    let iomem = procfs::iomem().expect("Can't read /proc/iomem");
