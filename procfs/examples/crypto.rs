use std::env::args;

use procfs::crypto;

pub fn main() {
    let crypto = crypto().expect("Was not able to access current crypto");
    let name_arg = args().nth(1);
