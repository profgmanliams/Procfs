use procfs::{prelude::*, CpuPressure, IoPressure, MemoryPressure, PressureRecord};

/// A basic example of /proc/pressure/ usage.
fn main() {
    if let Ok(pressure) = MemoryPressure::current() {
        println!("Memory Pressure:");
        println!("{:>10}:", "Some");
