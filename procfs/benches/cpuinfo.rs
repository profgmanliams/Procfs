use criterion::{black_box, criterion_group, criterion_main, Criterion};
use procfs::CpuInfo;

fn bench_cpuinfo(c: &mut Criterion) {
    c.bench_function("CpuInfo::new", |b| b.iter(|| black_box(CpuInfo::new().unwrap())));

