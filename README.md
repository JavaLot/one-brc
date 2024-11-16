# 1️⃣🐝🏎️ The One Billion Row Challenge

**Gunnar Morling** blog: [The One Billion Row Challenge](https://www.morling.dev/blog/one-billion-row-challenge/).
*Posted at Jan 1, 2024*

Source code on [GitHub](https://github.com/gunnarmorling/1brc)

Rust Discussion on [GitHub](https://github.com/gunnarmorling/1brc/discussions/57).

The challenge seemed interesting for learning a new programming language and
I decided to make an implementation in Rust.

## Rules and limits

- The computation must happen at application runtime, i.e. you cannot process the measurements file at build time.
- Input value ranges are as follows:
  - Station name: non-null `UTF-8` string of min length **1** character and max **length** 100 bytes, containing neither `;` nor `\n` characters. (i.e. this could be 100 one-byte characters, or 50 two-byte characters, etc.).
  - Temperature value: non-null double between **-99.9** (inclusive) and **99.9** (inclusive), always with **one fractional digit**.
- There is a maximum of `10,000` unique station names.
- Line endings in the file are `\n` characters on all platforms.
- Implementations must not rely on specifics of a given data set, e.g. any valid station name as per the constraints above and any data distribution (number of measurements per station) must be supported.
- The rounding of output values must be done using the semantics of IEEE 754 rounding-direction "roundTowardPositive".

### Result

The program should print out the min, mean, and max values per station, alphabetically ordered like so:

```
{Abha=5.0/18.0/27.4, Abidjan=15.7/26.0/34.1,  ...}
```

## My solution

### Data preparation

To create the initial data file, I made a list of cities with randomly generated
values of average temperatures [stations2.txt](stations2.txt)
and a conversion program [src/bin/generator.rs](src/bin/generator.rs).
The main parameters are set in the constants block:

```rust
const REQUIRED_KEYS: usize = 10_000;
const LINES_IN_TARGET: usize = 1_000_000_000;
const TARGET_FILE_NAME: &str = "measurements.txt";
const STATION_PATH: &str = "stations2.txt";
```

The format of the `stations2.txt` file is the same as in the requirements for the `measurements.txt`.

After generating `measurements.txt` with parameters above, it's size is about **17GB**.

```shell
cargo run --release --bin generator
```

### Finding a solution

**The first idea** is to read from a file in large blocks and pass them to other 
threads for processing, as they finish, aggregating the results. 
The example [examples/read-file-timing.rs](examples/read-file-timing.rs) partially implements this plan.

At implementation time, the library module [src/lib.rs](src/lib.rs) is drawn out . 
Common structures and functions with the best execution time fall into it. 
The main speed gain is given by the [ahash](https://docs.rs/ahash/latest/ahash/) library, 
measurements in [benches/stations-hash.rs](benches/stations-hash.rs). 
The function for converting numbers to temperature values is also written, 
measurements and different algorithms in [benches/numbers.rs](benches/numbers.rs).

The first implementation has three disadvantages: low speed of reading a file, 
reading can only be sequential, and memory control is needed because the entire 
file is loaded into RAM.

All these three problems disappear with the help of one library [memmap](https://docs.rs/memmap/latest/memmap/).

**The second idea** is to subdivide a memory-mapped file into blocks, pass them 
to other threads for processing, and as they finish, aggregate the results. 
Implementation in [examples/mmap-timing.rs](examples/mmap-timing).
Examples have also been written to compare the speed of breaking a block into lines
[examples/mmap-split-count.rs](examples/mmap-split-count) with the standard library of the Rust language, and 
[examples/mmap-memchr-count.rs](examples/mmap-memchr-count) using the library [memchr](https://docs.rs/memchr/latest/memchr/).

Solution is ready, copy the `main` function from [examples/mmap-timing.rs](examples/mmap-timing)
into [src/main.rs](src/main.rs). 

**Voilà!**

### Running

The processing file is set in [src/lib.rs](src/lib.rs) separately for development and release runtime:

```rust
#[cfg(debug_assertions)]
pub const FILE_PATH: &str = "measurements-small.txt"; // Debug configuration
#[cfg(not(debug_assertions))]
pub const FILE_PATH: &str = "measurements.txt"; // Release configuration
```

My solution:
```shell
cargo run --release --bin one-brc
```

Measurements of file loading and processing time:
```shell
cargo run --release --example read-file-timing
```

Measurements of memory mapped file and processing time:
```shell
cargo run --release --example mmap-timing
```

Example of counting lines in a file using `std::slice::split`:
```shell
cargo run --release --example mmap-split-count
```

Example of counting lines in a file using library `memchr`:
```shell
cargo run --release --example mmap-memchr-count
```

To run the benchmarks, we need to set the runtime for the project to `nightly`
in the file [rust-toolchain.toml](rust-toolchain.toml)

```toml
[toolchain]
channel = "nightly"
```

### Available hardware

| Name    | Cores/HT | OS        | RAM  | Disk |
|---------|----------|-----------|------|------|
| X86-64  | 8/16     | ArchLinux | 32GB | SSD  |
| Aarch64 | 10       | MacOS     | 32GB | SSD  |


### Compare Results

| Git Nik             | Arch/Lines/Keys | Result (5 best / first)          | Lang |
|---------------------|-----------------|----------------------------------|------|
| JavaLot             | X86-64/1B/10K   | 3.543229527s / 6.4570514s        | Rust |
| JavaLot             | Aarch64/1B/10K  | 3.389920542s / 16.526118166s     | Rust |
| k0nserv             | Aarch64/1B/10K  | 18.842061917s                    | Rust |
| tumdum              | X86-64/1B/10K   | 5.218078836s                     | Rust |
| tumdum              | Aarch64/1B/10K  | 3.607328959s                     | Rust |
| PurpleMyst          | X86-64/1B/10K   | N/A (Crash)                      | Rust |
| PurpleMyst          | Aarch64/1B/10K  | 68.030236791s                    | Rust |
| artsiomkorzun       | X86-64/1B/10K   | PT5.395278048S / PT16.14008725S  | Java |
| artsiomkorzun       | Aarch64/1B/10K  | PT1.838813792S / PT14.961604542S | Java |
| thomaswue           | X86-64/1B/10K   | PT5.215452171S                   | Java |
| thomaswue           | Aarch64/1B/10K  | PT2.23881425S                    | Java |
| shipilev            | X86-64/1B/10K   | PT6.1909899S                     | Java |
| shipilev            | Aarch64/1B/10K  | PT3.263435792S                   | Java |
| benhoyt             | X86-64/1B/10K   | 10 - 8.267941176s                | Go   |
| benhoyt             | Aarch64/1B/10K  | 10 - 5.421377209s                | Go   |
| benhoyt             | Aarch64/1B/10K  | 9 - 4.236911041s                 | Go   |
| AlexanderYastrebov  | X86-64/1B/10K   | 15.740465298s                    | Go   |
| AlexanderYastrebov  | Aarch64/1B/10K  | 4.574705167s                     | Go   |

### Time measurement

All runtime measurements were made from code by adding, for each language, a 
few lines to the beginning and end of the `main` function.
The number of runs for each solution is at least 5.

In Rust:
```rust
let start = Instant::now();
// ...
eprintln!("elapsed: {:?}", start.elapsed());
```

In Java:
```java
long start = System.nanoTime();
// ...
System.err.println(Duration.ofNanos(System.nanoTime() - start));
```
In Go:
```go
start := time.Now()
// ...
elapsed := time.Since(start)
fmt.Println(elapsed)
```

## Conclusions

In reality, no one will ever run the same program five times with the same data. 
In order to estimate throughput correctly, it is better to run it for different 
files. For this experiment I generated a couple more a billion lines files 
with 3000 and 15000 keys.

| Size  | Keys  | Result       |
|-------|-------|--------------|
| 16G   | 3000  | 5.750489644s |
| 17G   | 10000 | 6.390893488s |
| 17G   | 15000 | 7.183049551s |

It can be seen that for each new data file, its processing time is closer 
to the first run. On the other hand, the best time after five times of 
execution will be closer to the actual running time of the processing algorithm.

## What's next?

I would like to improve the block processing time, but I don't have a concrete idea yet, 
I'll wait, maybe it will mature.

## Acknowledgements

Many thanks to Gunnar Morling for sharing this challenge.
Thanks to the rustacean community for the wonderful language Rust and libraries.
