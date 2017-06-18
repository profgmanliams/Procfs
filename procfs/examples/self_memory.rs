use procfs::process::Process;

fn main() {
    let me = Process::myself().expect("Unable to load myself!");
    println!("PID: {}", me.pid);

    let page_size = procfs::page_size();
    println!("Memory page size: {}", page_size);

    // Note: when comparing the below values to what "top" will display, note that "top" will use
    // base-2 units (kibibytes), not base-10 units (kilobytes).

    if let Ok(stat) = me.stat() {
        println!("== Data from /proc/self/stat:");
        println!("Total virtual memory used: {} bytes", stat.vsize);
        println!(
            "Total resident set: {} pages ({} bytes)",
            stat.rss,
            stat.rss * page_size
        );
        println!();
    }

    if let Ok(statm) = me.statm() {
        println!("== Data from /proc/self/statm:");
        println!(
            "Total virtual memory used: {} pages ({} bytes)",
            statm.size,
