use procfs::prelude::*;
use std::collections::HashMap;

fn print(name: &str, indent: usize, mods: &HashMap<&str, Vec<&str>>) {
    println!("{}{} {}", if indent == 0 { "-" } else { " " }, " ".repeat(indent), name);

    if let Some(uses_list) = mods.get(name) {
        for name in uses_list {
            print(name, indent + 2, mods);
        }
    }
}

fn main() {
    let procfs::KernelModules(modules) = Current::current().unwrap();
