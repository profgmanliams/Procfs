#![allow(clippy::print_literal)]

extern crate procfs;

/// A very basic clone of `ps` on Linux, in the simple no-argument mode.
/// It shows all the processes that share the same tty as our self

fn main() {
