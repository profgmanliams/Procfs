// List mountpoints listed in /proc/mounts

fn main() {
let width = 15;
    for mount_entry in procfs::mounts().unwrap() {
        println!("Device: {}", mount_entry.fs_spec);
        println!("{:>width$}: {}", "Mount point", mount_entry.fs_file);
        println!("{:>width$}: {}","FS type", mount_entry.fs_vfstype);
        println!("{:>width$}: {}", "Dump", mount_entry.fs_freq);
