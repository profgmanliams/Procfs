procfs
======

[![Crate](https://img.shields.io/crates/v/procfs.svg)](https://crates.io/crates/procfs)
[![Docs](https://docs.rs/procfs/badge.svg)](https://docs.rs/procfs)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.70+-lightgray.svg)](https://github.com/eminence/procfs#minimum-rust-version)


This crate is an interface to the `proc` pseudo-filesystem on linux, which is normally mounted as `/proc`.
Long-term, this crate aims to be fairly feature complete, but at the moment not all files are exposed.
See the docs for info on what's supported, or view the [support.md](https://github.com/eminence/procfs/blob/master/support.md)
file in the code repository.

## Examples
There are several examples in the docs and in the [examples folder](https://github.com/eminence/procfs/tree/master/procfs/examples)
of the code repository.

Here's a small example that prints out all processes that are running on the same tty as the calling
process.  This is very similar to what "ps" does in its default mode:

```rust
fn main() {
    let me = procfs::process::Process::myself().unwrap();
    let me_stat = me.stat().unwrap();
    let tps = procfs::ticks_per_second();

    println!("{: >5} {: <8} {: >8} {}", "PID", "TTY", "TIME", "CMD");

    let tty = format!("pty/{}", me_stat.tty_nr().1);
    for prc in procfs::process::all_processes().unwrap() {
        let prc = prc.unwrap();
        let stat = prc.stat().unwrap();
        if stat.tty_nr == me_stat.tty_nr {
            // total_time is in seconds
            let total_time =
                (stat.utime + stat.stime) as f32 / (tps as f32);
