#[macro_use]
extern crate bencher;

use bencher::Bencher;
use systemstat::{Duration, Platform, System};

#[path = "../stats.rs"]
mod stats;
use stats::*;

fn all_from(bench: &mut Bencher) {
    let sys = System::new();
    bench.iter(|| AllStats {
        general: GeneralStats::from(&sys),
        cpu: CpuStats::from(&sys, Duration::from_millis(0)),
        memory: MemoryStats::from(&sys),
        filesystems: MountStats::from(&sys),
        network: NetworkStats::from(&sys),
    });
}

fn all_update(bench: &mut Bencher) {
    let sys = System::new();
    let mut stats = AllStats {
        general: GeneralStats::from(&sys),
        cpu: CpuStats::from(&sys, Duration::from_millis(0)),
        memory: MemoryStats::from(&sys),
        filesystems: MountStats::from(&sys),
        network: NetworkStats::from(&sys),
    };
    bench.iter(|| {
        stats.update(&sys, Duration::from_millis(0));
    });
}

fn general_from(bench: &mut Bencher) {
    let sys = System::new();
    bench.iter(|| GeneralStats::from(&sys));
}

fn general_update(bench: &mut Bencher) {
    let sys = System::new();
    let mut stats = GeneralStats::from(&sys);
    bench.iter(|| {
        stats.update(&sys);
    });
}

fn cpu_from(bench: &mut Bencher) {
    let sys = System::new();
    bench.iter(|| CpuStats::from(&sys, Duration::from_millis(0)));
}

fn cpu_update(bench: &mut Bencher) {
    let sys = System::new();
    let mut stats = CpuStats::from(&sys, Duration::from_millis(0));
    bench.iter(|| {
        stats.update(&sys, Duration::from_millis(0));
    });
}

fn net_from(bench: &mut Bencher) {
    let sys = System::new();
    bench.iter(|| NetworkStats::from(&sys));
}

fn net_update(bench: &mut Bencher) {
    let sys = System::new();
    let mut stats = NetworkStats::from(&sys);
    bench.iter(|| {
        stats.update(&sys);
    });
}

benchmark_group!(
    benches,
    all_from,
    all_update,
    general_from,
    general_update,
    cpu_from,
    cpu_update,
    net_from,
    net_update
);
benchmark_main!(benches);
