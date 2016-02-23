//! For each interface, display the number of bytes sent and received, along with a data rate

fn main() {
    let delay = std::time::Duration::from_secs(2);

    let mut prev_stats = procfs::net::dev_status().unwrap();
    let mut prev_now = std::time::Instant::now();
    loop {
        std::thread::sleep(delay);
        let now = std::time::Instant::now();
