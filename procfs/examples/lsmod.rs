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

    // each module has a list of what other modules use it.  Let's invert this and create a list of the modules used by each module.
    // This maps a module name to a list of modules that it uses
