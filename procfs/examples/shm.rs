extern crate procfs;
use procfs::prelude::*;

/// List processes using posix shared memory segments

fn main() {
    let shared_memory_vec = procfs::SharedMemorySegments::current().unwrap();

    for shared_memory in &shared_memory_vec.0 {
        println!("key: {}, shmid: {}", shared_memory.key, shared_memory.shmid);
        println!("============");

        for prc in procfs::process::all_processes().unwrap() {
            let prc = prc.unwrap();
            match prc.smaps() {
