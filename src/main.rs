use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use std::{thread, thread::available_parallelism};
use std::time::Instant;
use ahash::AHashMap;
use memmap::Mmap;
use one_brc::{FILE_PATH, LINE_MAX_LEN, process_block, result::TemperStatResult};

fn main() {
    let start = Instant::now();

    let cpu = available_parallelism().unwrap();
    let file = File::open(Path::new(FILE_PATH)).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let len = mmap.len();
    let size = len / cpu + LINE_MAX_LEN;

    thread::scope(|s| {
        let mut rest = mmap.as_ref();

        let mut threads= AHashMap::new();

        let (tx, rx) = channel::<i32>();

        let mut result = TemperStatResult::new();

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
                    (map, lines, errors)
                }
            });
            threads.insert(id, h);
            rest = tail;
            id += 1;
        }

        while !threads.is_empty() {
            let id = rx.recv().unwrap();
            if let Some(h) = threads.remove(&id) {
                if let Ok((map, _, _)) = h.join() {
                    result.aggregate(&map);
                }
            }
        }

        println!("{result}");
    });

    eprintln!("elapsed: {:?}", start.elapsed());
}
