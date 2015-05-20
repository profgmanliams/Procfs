use procfs::{prelude::*, CpuPressure, IoPressure, MemoryPressure, PressureRecord};

/// A basic example of /proc/pressure/ usage.
fn main() {
    if let Ok(pressure) = MemoryPressure::current() {
        println!("Memory Pressure:");
        println!("{:>10}:", "Some");
        print_pressure(pressure.some, 20);
        println!("{:>10}:", "Full");
        print_pressure(pressure.full, 20);
    }
    if let Ok(pressure) = CpuPressure::current() {
