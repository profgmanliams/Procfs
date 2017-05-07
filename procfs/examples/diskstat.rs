use procfs::{diskstats, process::Process, DiskStat};
use std::collections::HashMap;
use std::iter::FromIterator;

fn main() {
    let me = Process::myself().unwrap();
    let mounts = me.mountinfo().unwrap();

    // Get a list of all disks that we have IO stat info on
    let disk_stats: HashMap<(i32, i32), DiskStat> =
        HashMap::from_iter(diskstats().unwrap().into_iter().map(|i| ((i.major, i.minor), i)));

    for mount in mounts {
        // parse the majmin string (something like "0:3") into an (i32, i32) tuple
        let (maj, min): (i32, i32) = {
