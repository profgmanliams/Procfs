#![allow(clippy::print_literal)]

extern crate procfs;

/// A very basic clone of `ps` on Linux, in the simple no-argument mode.
/// It shows all the processes that share the same tty as our self

fn main() {
    let mestat = procfs::process::Process::myself().unwrap().stat().unwrap();
    let tps = procfs::ticks_per_second();

    println!("{: >10} {: <8} {: >8} {}", "PID", "TTY", "TIME", "CMD");

    let tty = format!("pty/{}", mestat.tty_nr().1);
    for p in procfs::process::all_processes().unwrap() {
        let prc = p.unwrap();
