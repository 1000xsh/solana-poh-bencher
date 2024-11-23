# poh-bencher
wip

a lightweight tool to benchmark cpu cores for proof of history (poh).

## features
- benchmark specific cores with cpu affinity.
- find the best-performing core automatically.
- customizable benchmark duration and batch size.

## usage
```bash
# build
cargo b -r

# run directly
cargo run --release -- --find-best-core --time 5 --samples 1000000

# list available cores
poh-bencher --list-cores

# benchmark a specific core
poh-bencher --core 0 --time 10 --samples 1000000

# find the best-performing core
poh-bencher --find-best-core --time 10 --samples 1000000
