use procfs::process::{all_processes, Stat};

struct ProcessEntry {
    stat: Stat,
    cmdline: Option<Vec<String>>,
}

/// Print all processes as a tree.
/// The tree reflects the hierarchical relationship between parent and child processes.
fn main() {
    // Get all processes
    let processes: Vec<ProcessEntry> = match all_processes() {
        Err(err) => {
            println!("Failed to read all processes: {}", err);
