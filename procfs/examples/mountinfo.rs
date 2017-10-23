use procfs::process::Process;
use std::collections::HashSet;

fn main() {
    for mount in Process::myself().unwrap().mountinfo().unwrap() {
        let (a, b): (HashSet<_>, HashSet<_>) = mount
            .mount_options
