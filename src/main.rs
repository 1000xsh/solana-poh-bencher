use clap::{Arg, ArgAction, Command};
use core_affinity;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

fn compute_hash_time(samples: u64) -> Duration {
    let mut hasher = Sha256::new();
    let start = Instant::now();
    for _ in 0..samples {
        hasher.update(rand::random::<u64>().to_le_bytes());
        let _ = hasher.finalize_reset();
    }
    start.elapsed()
}

fn set_cpu_affinity_for_core(core: core_affinity::CoreId) -> Result<(), String> {
    if core_affinity::set_for_current(core) {
        Ok(())
    } else {
        Err("failed to set cpu affinity for the core.".to_string())
    }
}

fn compute_poh_statistics(
    benchmark_time: Duration,
    samples_to_test: u64,
) -> (u64, Duration, Duration, Duration) {
    let mut total_hashes = 0;
    let mut total_time = Duration::ZERO;
    let mut best_latency = Duration::MAX;
    let mut worst_latency = Duration::ZERO;

    let start_time = Instant::now();
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
    }

    (total_hashes, total_time, best_latency, worst_latency)
}

fn check_poh_speed(
    core_id: &core_affinity::CoreId,
    benchmark_time: Duration,
    hash_samples: Option<u64>,
) -> Result<u64, String> {
    set_cpu_affinity_for_core(core_id.clone())?;
    // fix me
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

    println!("core tested: {:?}", core_id);
    println!("avg latency per batch: {:?}", avg_latency);
    println!("best latency: {:?}", best_latency);
    println!("worst latency: {:?}", worst_latency);

    println!("results:");
    println!("total hashes computed: {}", total_hashes);
    println!("benchmark duration: {:?}", total_time);
    println!("computed hashes per second: {}", hashes_per_second);
    println!("target hashes per second: {}", target_hashes_per_second);

    if hashes_per_second < target_hashes_per_second {
        Err(format!(
            "poh speed too slow: mine = {}, target = {}",
            hashes_per_second, target_hashes_per_second
        ))
    } else {
        Ok(hashes_per_second)
    }
}

fn find_best_core(benchmark_time: Duration, hash_samples: Option<u64>) {
    if let Some(cores) = core_affinity::get_core_ids() {
        let mut best_core = None;
        let mut best_hashes_per_second = 0;

        println!("benchmarking all available cores...");
        for core_id in &cores {
            println!("benchmarking core {:?}", core_id);
            match check_poh_speed(core_id, benchmark_time, hash_samples) {
                Ok(hashes_per_second) => {
                    if hashes_per_second > best_hashes_per_second {
                        best_core = Some(core_id);
                        best_hashes_per_second = hashes_per_second;
                    }
                }
                Err(err) => {
                    println!("failed on core {:?}: {}", core_id, err);
                }
            }
        }

        if let Some(core) = best_core {
            println!("best performing core: {:?}", core);
            println!("hashes per second: {}", best_hashes_per_second);
        } else {
            println!("no core performed successfully.");
        }
    } else {
        println!("failed to retrieve core ids.");
    }
}

fn main() {
    let matches = Command::new("poh bencher")
        .version("1.2")
        .author("1000x.sh <a@1000x.sh>")
        .about("benchmark cores for poh")
        .arg(
            Arg::new("core")
                .short('c')
                .long("core")
                .value_name("CORE")
                .help("specifies which core to test")
                .default_value("0"),
        )
        .arg(
            Arg::new("time")
                .short('t')
                .long("time")
                .value_name("SECONDS")
                .help("duration of the benchmark in seconds")
                .default_value("10"),
        )
        .arg(
            Arg::new("samples")
                .short('s')
                .long("samples")
                .value_name("SAMPLES")
                .help("number of hash samples per batch (optional)"),
        )
        .arg(
            Arg::new("list-cores")
                .short('l')
                .long("list-cores")
                .help("list available cores and exit")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("find-best-core")
                .short('f')
                .long("find-best-core")
                .help("benchmark all cores and find the best performing one")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if matches.get_flag("list-cores") {
        if let Some(cores) = core_affinity::get_core_ids() {
            println!("available cores:");
            for (i, core) in cores.iter().enumerate() {
                println!("core {}: {:?}", i, core);
            }
        } else {
            println!("failed to retrieve core ids.");
        }
        return;
    }

    let benchmark_time: u64 = matches
        .get_one::<String>("time")
        .unwrap()
        .parse()
        .expect("invalid time value");
    let samples = matches
        .get_one::<String>("samples")
        .map(|s| s.parse::<u64>().expect("invalid samples value"));

    if matches.get_flag("find-best-core") {
        find_best_core(Duration::from_secs(benchmark_time), samples);
        return;
    }

    if let Some(cores) = core_affinity::get_core_ids() {
        let core_index: usize = matches
            .get_one::<String>("core")
            .unwrap()
            .parse()
            .expect("invalid core value");

        if core_index >= cores.len() {
            eprintln!(
                "invalid core index: {}. system has {} cores.",
                core_index,
                cores.len()
            );
            return;
        }

        let core = &cores[core_index];
        match check_poh_speed(core, Duration::from_secs(benchmark_time), samples) {
            Ok(_) => println!("poh speed check passed!"),
            Err(err) => eprintln!("poh speed check failed: {}", err),
        }
    } else {
        println!("failed to retrieve core ids.");
    }
}
