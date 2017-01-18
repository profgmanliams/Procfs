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
            return;
        }
        Ok(processes) => processes,
    }
    .filter_map(|v| {
        v.and_then(|p| {
            let stat = p.stat()?;
            let cmdline = p.cmdline().ok();
            Ok(ProcessEntry { stat, cmdline })
        })
        .ok()
    })
    .collect();
    // Iterate through all processes and start with top-level processes.
    // Those can be identified by checking if their parent PID is zero.
