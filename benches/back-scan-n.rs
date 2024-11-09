#![feature(test)]

extern crate test;

use test::Bencher;
use memchr::memrchr;

#[cfg(test)]

const TEST_STR1: &str = "\
愛媛県今治市;20.8
Brussels;14.9
Lake Tekapo;6.6
Москва;16.7
";

#[test]
fn print_test_str() {
    let slice = TEST_STR1.as_bytes();
    println!("{TEST_STR1}");
    println!("{}\t{:?}\n", slice.len(), slice);
}

#[inline]
fn back_scan_n_iter(buf: &[u8]) -> usize {
    buf.iter().rev().position(|&last_line| last_line == b'\n').unwrap_or_else(|| 0)
}

#[inline]
fn back_scan_n_pointer(buf: &[u8]) -> usize {
    let l = buf.len();
    if l == 0 { return 0 }
    let mut p_cur = unsafe{ buf.as_ptr().add(l - 1)};
    let mut pos: usize = 0;
    while pos < l {
        if unsafe { *p_cur } != b'\n' {
            pos += 1;
            p_cur = unsafe{ p_cur.sub(1) }
        } else { return pos }
    }
    0
}

#[inline]
fn back_scan_n_memchr(buf: &[u8]) -> usize {
    let res = memrchr(b'\n', buf);
    match res {
        Some(pos) => buf.len() - pos - 1,
        None => 0
    }
}

#[test]
fn test_back_scans() {
    let len = TEST_STR1.len();
    for i in 0..len {
        let slice = &TEST_STR1.as_bytes()[..len - i];
        let pos_iter = back_scan_n_iter(slice);
        let pos_pointer = back_scan_n_pointer(slice);
        let pos_memchr = back_scan_n_memchr(slice);
        assert_eq!(pos_iter, pos_pointer);
        assert_eq!(pos_pointer, pos_memchr);
        println!("{}\t{pos_iter}\t{pos_pointer}\t{pos_memchr}\t{:?}", slice.len(), slice);
    }
}

fn back_scan_iter(buf: &[u8]) -> usize {
    let len = buf.len();
    let mut sum: usize = 0;
    for i in 0..len {
        let slice = &buf[..len - i];
        sum += back_scan_n_iter(slice);
    }
    sum
}

fn back_scan_pointer(buf: &[u8]) -> usize {
    let len = buf.len();
    let mut sum: usize = 0;
    for i in 0..len {
        let slice = &buf[..len - i];
        sum += back_scan_n_pointer(slice);
    }
    sum
}

fn back_scan_memchr(buf: &[u8]) -> usize {
    let len = buf.len();
    let mut sum: usize = 0;
    for i in 0..len {
        let slice = &buf[..len - i];
        sum += back_scan_n_memchr(slice);
    }
    sum
}

#[bench]
fn bench_back_scan_iter(b: &mut Bencher) {
    let buf = TEST_STR1.as_bytes();
    b.iter(|| back_scan_iter(buf));
}

#[bench]
fn bench_back_scan_pointer(b: &mut Bencher) {
    let buf = TEST_STR1.as_bytes();
    b.iter(|| back_scan_pointer(buf));
}

#[bench]
fn bench_back_scan_memchr(b: &mut Bencher) {
    let buf = TEST_STR1.as_bytes();
    b.iter(|| back_scan_memchr(buf));
}
