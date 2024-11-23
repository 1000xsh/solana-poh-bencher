use crate::cpu_affinityx::{get_cpu_affinityx, set_cpu_affinityx};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

pub fn compute_hash_time(samples: u64) -> Duration {
    let mut hasher = Sha256::new();
    let start = Instant::now();
    for _ in 0..samples {
        hasher.update(rand::random::<u64>().to_le_bytes());
        let _ = hasher.finalize_reset();
    }
    start.elapsed()
}

pub fn compute_poh_statistics(
    benchmark_time: Duration,
    samples_to_test: u64,
) -> (u64, Duration, Duration, Duration) {
    let mut total_hashes = 0;
    let mut total_time = Duration::ZERO;
    let mut best_latency = Duration::MAX;
    let mut worst_latency = Duration::ZERO;

    let start_time = Instant::now();
    let progress_bar = ProgressBar::new(benchmark_time.as_secs());
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{elapsed_precise}] [{wide_bar}] {percent}%")
            .expect("failed to set progress bar template")
            .progress_chars("█  "),
    );

    while start_time.elapsed() < benchmark_time {
        let start_hash_time = Instant::now();
        compute_hash_time(samples_to_test);
        let elapsed = start_hash_time.elapsed();

        total_hashes += samples_to_test;
        total_time += elapsed;

        if elapsed < best_latency {
            best_latency = elapsed;
        }
        if elapsed > worst_latency {
            worst_latency = elapsed;
        }

        // update progress bar with current hash speed and latency
        let elapsed_secs = start_time.elapsed().as_secs();
        let hashes_per_second = if elapsed_secs > 0 {
            total_hashes / elapsed_secs
        } else {
            0
        };

        let message = format!(
            "benchmarking... | hash rate: {} h/s | latency: {:?}",
            hashes_per_second, elapsed
        );
        progress_bar.set_message(message);
        progress_bar.set_position(elapsed_secs.min(benchmark_time.as_secs()));
    }

    progress_bar.finish_with_message("benchmark completed.");
    (total_hashes, total_time, best_latency, worst_latency)
}

pub fn check_poh_speed(
    core_index: usize,
    benchmark_time: Duration,
    hash_samples: Option<u64>,
) -> Result<u64, String> {
    // set cpu affinity directly using the provided core index
    set_cpu_affinityx(core_index)?;

    // verify the affinity
    let current_affinity = get_cpu_affinityx()?;
    if !current_affinity.contains(&core_index) {
        return Err(format!(
            "failed to bind process to core {}. current affinity: {:?}",
            core_index, current_affinity
        ));
    }

    let hashes_per_tick = 10_000;
    let ticks_per_slot = 64;
    let ns_per_slot = 400_000_000;

    let hashes_per_slot = hashes_per_tick * ticks_per_slot;
    let samples_to_test = hash_samples.unwrap_or(hashes_per_slot as u64);

    let (total_hashes, total_time, best_latency, worst_latency) =
        compute_poh_statistics(benchmark_time, samples_to_test);

    let avg_latency = total_time / (total_hashes / samples_to_test) as u32;
    let hashes_per_second = total_hashes / total_time.as_secs();

    let target_slot_duration = Duration::from_nanos(ns_per_slot as u64);
    let target_hashes_per_second =
        (hashes_per_slot as f64 / target_slot_duration.as_secs_f64()) as u64;

    println!("\n=== benchmark results ===");
    println!("core tested:            {}", core_index);
    println!("current affinity:       {:?}", current_affinity);
    println!("-------------------------------");
    println!("total hashes computed:  {}", total_hashes);
    println!("benchmark duration:     {:?}", total_time);
    println!("hashes per second:      {}", hashes_per_second);
    println!("target hashes per second: {}", target_hashes_per_second);
    println!("-------------------------------");
    println!("avg latency per batch:  {:?}", avg_latency);
    println!("best latency:           {:?}", best_latency);
    println!("worst latency:          {:?}", worst_latency);

    if hashes_per_second < target_hashes_per_second {
        Err(format!(
            "poh speed too slow: mine = {}, target = {}",
            hashes_per_second, target_hashes_per_second
        ))
    } else {
        println!("poh speed check passed.");
        Ok(hashes_per_second)
    }
}
