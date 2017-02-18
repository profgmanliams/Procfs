use criterion::{black_box, criterion_group, criterion_main, Criterion};
use procfs::CpuInfo;

fn bench_cpuinfo(c: &mut Criterion) {
