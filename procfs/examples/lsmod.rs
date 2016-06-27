use procfs::prelude::*;
use std::collections::HashMap;

fn print(name: &str, indent: usize, mods: &HashMap<&str, Vec<&str>>) {
    println!("{}{} {}", if indent == 0 { "-" } else { " " }, " ".repeat(indent), name);

