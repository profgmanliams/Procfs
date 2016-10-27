// List mountpoints listed in /proc/mounts

fn main() {
let width = 15;
    for mount_entry in procfs::mounts().unwrap() {
        println!("Device: {}", mount_entry.fs_spec);
