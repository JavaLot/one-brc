use std::time::Instant;
use one_brc::temp_round;

#[test]
fn test_f32_million_error() {
    const MAX_TEMP: f32 = 99.9f32;
    const N: usize = 1_190_000;
    let mut sum: f32 = 0.0;

    for _ in 0..N {
        sum += MAX_TEMP;
    }

    let mean = sum / N as f32;
    let round = temp_round(mean as f64);
    assert_ne!(MAX_TEMP, round);
    assert_eq!(diff_pos(MAX_TEMP as f64, mean as f64), 0);
}

#[test]
fn test_f64_billion() {
    const MAX_TEMP: f64 = 99.9;
    const N: usize = 1_000_000_000;
    let mut sum = 0.0;

    for _ in 0..N {
        sum += MAX_TEMP;
    }

    let mean = sum / N as f64;
    let round = temp_round(mean);
    assert!(diff_pos(MAX_TEMP, mean) > 0);
    assert_eq!(MAX_TEMP as f32, round);
}

#[test]
fn test_f32_to_f64_billion() {
    let start = Instant::now();
    const MAX_TEMP: f32 = 99.9;
    const N: usize = 1_000_000_000;
    let mut sum: f64 = 0.0;

    for _ in 0..N {
        sum += MAX_TEMP as f64;
    }

    let mean = sum / N as f64;
    let round = temp_round(mean);
    assert!(diff_pos(MAX_TEMP as f64, mean) > 0);
    assert_eq!(MAX_TEMP, round);
    println!("{:?}", start.elapsed());
}

#[test]
/// Rust Round to nearest, ties away from zero (or ties to away) – rounds to the nearest value;
/// if the number falls midway, it is rounded to the nearest value above
/// (for positive numbers) or below (for negative numbers).
fn test_rust_temp_rounding() {
    use rust_decimal::{Decimal, RoundingStrategy};

    fn out(n: i32) {
        let f_64 = n as f64 / 100.0;
        let f_32 = n as f32 / 100.0;
        let d = Decimal::from_f64_retain(f_64).unwrap().round_dp_with_strategy(1, RoundingStrategy::ToPositiveInfinity);
        let m = temp_round(f_64);
        println!("N: {n},\tf64: {f_64:.1},\tf32: {f_32:.1},\torigin: {f_64}\tdecimal: {d}, \tone-brc: {m:.1}");
    }

    for n in 230..=241 {
        out(n);
    }
    for n in -241..=-229 {
        out(n);
    }
}

/// calculate the difference position in numbers after digital point
fn diff_pos(n1: f64, n2: f64) -> u8 {
    let mut count: u8 = 0;
    let mut mul = 1.0;
    while (n1 * mul).trunc() == (n2 * mul).trunc() {
        count += 1;
        mul *= 10.0;
    }
    count
}
