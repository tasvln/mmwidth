use prettytable::{Table, row};

mod helpers;

// Buffer sizes chosen to land squarely inside each cache tier.
// Adjust to match your CPU's actual cache sizes (check `lscpu` or `sysctl`).
// const TIERS: &[(&str, usize)] = &[
//     ("L1", 16 * 1024),          //  16 KB  (L1 is usually 32–64 KB per core)
//     ("L2", 512 * 1024),         // 512 KB  (L2 is usually 256 KB – 1 MB)
//     ("L3", 8 * 1024 * 1024),    //   8 MB  (L3 is usually 6–32 MB)
//     ("RAM", 256 * 1024 * 1024), // 256 MB  (guaranteed to spill past all caches)
// ];

// My M1 mac has a unified memory ran and not an L3
const TIERS: &[(&str, usize)] = &[
    ("L1", 64 * 1024),          //  64 KB  — half of P-core L1 (128 KB)
    ("L2", 6 * 1024 * 1024),    //   6 MB  — half of P-core L2 (12 MB)
    ("RAM", 256 * 1024 * 1024), // 256 MB  — way past all caches
];

fn main() {
    println!("\nCPU Memory Bandwidth Profiler\n");

    let mut table = Table::new();

    table.add_row(row![
        bFc->"Tier",
        bFc->"Buffer",
        bFc->"Stream BW (GB/s)",
        bFc->"Chase latency (ns)"
    ]);

    for &(name, buf_bytes) in TIERS {
        let stream_bw = helpers::bench_stream(buf_bytes);
        let chase_ns = helpers::bench_chase(buf_bytes);
        table.add_row(row![
            name,
            helpers::format_size(buf_bytes),
            format!("{:.1}", stream_bw),
            format!("{:.1}", chase_ns),
        ]);
    }

    table.printstd();
    println!();
    println!("Stream -> sequential 64-bit reads; compiler barriers prevent");
    println!("          dead-code elimination. Measures pure bandwidth.");
    println!("Chase : random-order linked list traversal; each access");
    println!("        depends on the previous result, defeating prefetch.");
    println!("        Measures true round-trip latency per cache level.");
    println!();
}
