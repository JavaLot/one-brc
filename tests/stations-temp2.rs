use std::time::Instant;

#[test]
fn test_i16_to_i64_billion() {
    let start = Instant::now();
    const MAX_TEMP: i16 = 999;
    const N: usize = 1_000_000_000;
    let mut sum: i64 = 0;

    for _ in 0..N {
        sum += MAX_TEMP as i64;
    }

    let mean = sum as f64 / (N * 10) as f64;
    println!("{:?},\t{mean}", start.elapsed());
    assert_eq!((mean * 10.0).trunc() as i16, MAX_TEMP);
}

#[test]
fn numbers_mapping() {
    for i in -999..=999 {
        let f = format!("{:.1}", (i as f32) / 10.0);
        let b = f.as_bytes();
        println!("{i}\t{f}\t{b:?}");
    }
}