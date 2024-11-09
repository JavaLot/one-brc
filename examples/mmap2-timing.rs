use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::available_parallelism;
use std::time::{Duration, Instant};
use ahash::AHashMap;
use memmap2::Mmap;
use one_brc::{FILE_PATH, LINE_MAX_LEN, process_block};
use one_brc::result::TemperStatResult;

fn main() {
    let start = Instant::now();
    let cpu = available_parallelism().unwrap();
    let file = File::open(Path::new(FILE_PATH)).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let len = mmap.len();
    let size = len / cpu + LINE_MAX_LEN;
    eprintln!("file len: {len}, block size: {size}");

    thread::scope(|s| {
        let mut rest = mmap.as_ref();

        let mut threads= AHashMap::new();

        let (tx, rx) = channel::<i32>();

        let mut result = TemperStatResult::new();
        let mut lines: usize = 0;
        let mut errors: usize = 0;
        let mut total_aggregate = Duration::default();
        let mut min_time = Duration::new(1000, 0);
        let mut max_time = Duration::default();

        let mut id = 0;
        while !rest.is_empty() {
            let mut l = if rest.len() > size { size } else { rest.len() };
            let mut i = l;
            while i > 0 && rest[i - 1] != b'\n' {
                i -= 1;
            }
            if i != 0 { l = i };
            let (cur, tail) = rest.split_at(l);

            let h = s.spawn({
                let tx = tx.clone();
                move || {
                    let (map, lines, errors) = process_block(cur, 7000);
                    tx.send(id).unwrap();
                    (Instant::now(), map, lines, errors)
                }
            });
            threads.insert(id, (h, Instant::now()));
            rest = tail;
            id += 1;
        }

        while !threads.is_empty() {
            let id = rx.recv().unwrap();
            if let Some((h, started)) = threads.remove(&id) {
                if let Ok((finished, map, l, e)) = h.join() {
                    let wait = started.duration_since(start);
                    let time = finished.duration_since(started);
                    let aggregate = Instant::now();
                    result.aggregate(&map);
                    lines += l;
                    errors += e;
                    let d = aggregate.elapsed();
                    total_aggregate += d;
                    min_time = time.min(min_time);
                    max_time = time.max(max_time);
                    eprintln!("{id}\tmap len: {},\tlines: {l},\terrors: {e},\tawait:  {:?},\ttime {:?},\taggregate {:?}", map.len(), wait, time, d);
                }
            }
        }

        let start_print = Instant::now();
        println!("{result}");
        eprintln!("print duration: {:?}\ttotal lines: {lines},\ttotal errors: {errors},\ttotal aggregate: {total_aggregate:?},\tmin time: {min_time:?},\tmax time: {max_time:?}", start_print.elapsed());
    });

    eprintln!("elapsed: {:?}", start.elapsed());
}

#[test]
fn test() {
    use std::sync::{Arc, Mutex};

    let start = Instant::now();

    let data = Arc::new(Mutex::new(String::new()));

    thread::scope( |s| {
        for i in 0..10 {
            s.spawn({
                let data = Arc::clone(&data);
                move || {
                    let id = thread::current().id();
                    let mut g = data.lock().unwrap();
                    let s = format!("{i} - {id:?}, ");
                    g.push_str(&s);
                }
            });
        }
    });
    println!("{data:?}");
    println!("{:?}", start.elapsed())
}

#[test]
fn test_split_at_bounds() {
    let s = b"1234";
    for i in 0..= s.len() {
        let (l, r) = s.split_at(i);
        println!("{i}\t{:?} - {:?}", l, r);
    }
}