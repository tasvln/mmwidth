use std::time::Instant;

// 128 MB of reads per timing window
pub const STREAM_BYTES_PER_PASS: u64 = 1 << 27;

// repeat each test N times
pub const ITERATIONS: usize = 10;

pub fn bench_stream(buf_bytes: usize) -> f64 {
    // number of u64 elements
    let n = buf_bytes / std::mem::size_of::<u64>();

    // fill with 0,1,2,3... // how many full passes to reach ~128 MB of total reads
    let buf: Vec<u64> = (0..n as u64).collect();

    let passes = (STREAM_BYTES_PER_PASS as usize / buf_bytes).max(1);

    let mut best_bw: f64 = 0.0;

    for _ in 0..ITERATIONS {
        let mut sink: u64 = 0;
        // start the clock
        let t0 = Instant::now();

        for _ in 0..passes {
            for &v in buf.iter() {
                // Prevent the compiler from eliminating the read -> use every byte read
                sink = sink.wrapping_add(v);
            }
        }

        // Keep `sink` alive so the optimizer can't remove the loop.
        std::hint::black_box(sink);

        // prevent dead-code removal
        let elapsed = t0.elapsed().as_secs_f64();
        let bytes_read = (buf_bytes * passes) as f64;
        let bw = bytes_read / elapsed / 1e9;

        // keep the fastest run
        if bw > best_bw {
            best_bw = bw;
        }
    }

    // returned in GB/s
    best_bw
}

pub fn bench_chase(buf_bytes: usize) -> f64 {
    let n = buf_bytes / std::mem::size_of::<u64>();

    // Build a random permutation (Fisher-Yates) so node i → perm[i].
    let mut perm: Vec<usize> = (0..n).collect();

    // Deterministic LCG — no dependency on rand crate.
    // a cheap PRNG (pseudo-random number generator) that needs no external crate
    let mut rng: u64 = 0xdeadbeef_cafef00d;
    for i in (1..n).rev() {
        // two magic constants are from Knuth's table of well-tested LCG parameters
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        perm.swap(i, j);
    }

    // Encode permutation as a heap-allocated array of next-pointers.
    // buf[i] = index of the next node to visit.
    let mut buf: Vec<u64> = vec![0u64; n];
    for i in 0..n {
        buf[i] = perm[i] as u64;
    }
    let ptr = buf.as_ptr();

    let mut best_ns: f64 = f64::MAX;
    for _ in 0..ITERATIONS {
        let mut idx: u64 = 0;
        let t0 = Instant::now();

        // Walk all n nodes. Each iteration loads buf[idx], which gives next idx.
        // The data dependency forces serialisation — no out-of-order win.
        for _ in 0..n {
            // SAFETY: idx is always a valid index (permutation guarantees it).
            idx = unsafe { *ptr.add(idx as usize) };
        }
        std::hint::black_box(idx);

        let elapsed = t0.elapsed().as_secs_f64();
        let ns_per_access = elapsed * 1e9 / n as f64;
        if ns_per_access < best_ns {
            best_ns = ns_per_access;
        }
    }
    best_ns
}

pub fn format_size(bytes: usize) -> String {
    if bytes >= 1 << 20 {
        format!("{} MB", bytes >> 20)
    } else {
        format!("{} KB", bytes >> 10)
    }
}
