#![allow(clippy::print_literal)]

extern crate procfs;

use procfs::process::{FDTarget, Stat};

use std::collections::HashMap;

fn main() {
    // get all processes
    let all_procs = procfs::process::all_processes().unwrap();

    // build up a map between socket inodes and processes:
