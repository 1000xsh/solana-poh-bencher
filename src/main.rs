use clap::{Arg, Command};
use std::time::Duration;
mod bench;
mod cpu_affinityx;
use bench::check_poh_speed;

fn main() {
    let matches = Command::new("poh bencher")
        .version("1.3")
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
        .get_matches();

    let benchmark_time: u64 = matches
        .get_one::<String>("time")
        .unwrap()
        .parse()
        .expect("invalid time value");
    let samples = matches
        .get_one::<String>("samples")
        .map(|s| s.parse::<u64>().expect("invalid samples value"));

    let core_index: usize = matches
        .get_one::<String>("core")
        .unwrap()
        .parse()
        .expect("invalid core value");

    match check_poh_speed(core_index, Duration::from_secs(benchmark_time), samples) {
        Ok(_) => println!("poh speed check passed!"),
        Err(err) => eprintln!("poh speed check failed: {}", err),
    }
}
