use procfs::{diskstats, process::Process, DiskStat};
use std::collections::HashMap;
use std::iter::FromIterator;

fn main() {
    let me = Process::myself().unwrap();
    let mounts = me.mountinfo().unwrap();
