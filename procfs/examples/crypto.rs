use std::env::args;

use procfs::crypto;

pub fn main() {
    let crypto = crypto().expect("Was not able to access current crypto");
    let name_arg = args().nth(1);
    for (name, entries) in crypto.crypto_blocks {
        if let Some(ref name_find) = name_arg {
            if !name.contains(name_find) {
                continue;
            }
        }
        println!("Type: {name}");
