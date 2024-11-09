use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use one_brc::{FILE_PATH, process_block};
use one_brc::result::TemperStatResult;

/// Read file into memory
///
/// ### Parameters:
/// - `path` - `Path` to file
/// - `block` - Number of byte blocks the file will be read into
///
/// ### Return:
/// - `Vec<Vec<u8>>` - file blocks
/// - `u64` - file len
/// - `usize` - loops
/// - `usize` - read bytes
///
fn read_file(path: &Path, blocks: usize) -> (Vec<Vec<u8>>, u64, usize, usize) {
    let start = Instant::now();

    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
    );

    eprintln!("free/avail: {}/{} bytes", sys.free_memory(), sys.available_memory());

    let mut file = File::open(path).unwrap();
    let len = file.metadata().unwrap().len();
    let block_size = (len + 512) as usize / blocks;
    let mut rest = len as usize;

    let mut loops: usize = 0;
    let mut bytes: usize = 0;

    let mut bufs: Vec<Vec<u8>> = Vec::new();

    while rest > 0 {
        let need_len = if rest > block_size { block_size } else { rest };

        sys.refresh_memory();
        if (sys.available_memory() as usize) < (need_len * 3) {
            eprintln!("Need more memory!");
            break;
        }

        // alloc
        let start_alloc = Instant::now();
        let mut buf = Vec::<u8>::with_capacity(need_len);
        unsafe { buf.set_len(need_len); }
        let dur_alloc = start_alloc.elapsed();

        // read
        let start_read = Instant::now();
        let _ = file.read(buf.as_mut_slice()).unwrap(); // read_len
        let dur_read = start_read.elapsed();

        // find block line end offset
        let mut dur_seek= Duration::from_secs(0);

        let start_seek = Instant::now();
        if let Some(offset) = buf.iter().rev().position(|&nl| nl == b'\n')  {
            if offset > 0 {
                let _ = file.seek(SeekFrom::Current(-(offset as i64))); // seek_pos
                buf.truncate(buf.len() - offset);
                dur_seek = start_seek.elapsed();
            }
        }

        loops += 1;
        bytes += buf.len();
        rest -= buf.len();

        sys.refresh_memory();
        eprintln!("i: {loops}\tfree/avail: {}/{} bytes\talloc/read/seek: {:?}/{:?}/{:?}\tbuf len: {}", sys.free_memory(), sys.available_memory(), dur_alloc, dur_read, dur_seek, buf.len());

        bufs.push(buf);
    }

    eprintln!("Inside read_file: {:?}", start.elapsed());
    (bufs, len, loops, bytes)
}

fn main() {
    let start = Instant::now();

    let (bufs, len, loops, bytes) = read_file(Path::new(FILE_PATH), thread::available_parallelism().unwrap().get());

    eprintln!("File read: {:?}", start.elapsed());
    eprintln!("file length: {len}, loops: {loops}, read bytes: {bytes}");

    thread::scope(|scope|{
        let start_threads = Instant::now();

        let mut total_aggregate = Duration::default();
        let mut min_time = Duration::new(1000, 0);
        let mut max_time = Duration::default();

        let mut threads= Vec::new();

        bufs.iter().for_each(|block| {
            let handle = scope.spawn(move || {
                let (map, lines, errors) = process_block(block.as_slice(), 7000);
                (Instant::now(), map, lines, errors)
            });

            threads.push((handle, Instant::now()));
        });
        eprintln!("Threads for block processing started: {:?}", start_threads.elapsed());

        let start_joining = Instant::now();

        let mut result = TemperStatResult::new();
        let mut lines: usize = 0;
        let mut errors: usize = 0;

        threads.into_iter().for_each(|(h, started)| {
            if let Ok((finished, map, l, e)) = h.join() {
                let wait = started.duration_since(start_threads);
                let time = finished.duration_since(started);
                let aggregate = Instant::now();
                result.aggregate(&map);
                lines += l;
                errors += e;
                let d = aggregate.elapsed();
                total_aggregate += d;
                min_time = time.min(min_time);
                max_time = time.max(max_time);
                eprintln!("map len: {},\tlines: {l},\terrors: {e},\tawait:  {:?},\ttime {:?},\taggregate {:?}", map.len(), wait, time, d);
            }
        });
        eprintln!("Finish joining threads after block processing: {:?}", start_joining.elapsed());
        eprintln!("total lines: {lines},\ttotal errors: {errors}");
        let start_print = Instant::now();
        // println!("{result}");
        eprintln!("print duration: {:?}\ttotal lines: {lines},\ttotal errors: {errors},\ttotal aggregate: {total_aggregate:?},\tmin time: {min_time:?},\tmax time: {max_time:?}", start_print.elapsed());
    });

    eprintln!("Finish: {:?}", start.elapsed());
}

// --release 2.148353064s, lines: 60316011 AHashMap::new(), map capacity: 14336
// --release 2.127354298s, lines: 60316011 AHashMap::with_capacity(10_000), map capacity: 14336

#[test]
fn test_spawn_join() {
    use ahash::AHashMap;
    use one_brc::TemperStat;

    thread::scope(|s| {
        let str = "hello join";
        let h = s.spawn(move || {
            let mut ts = TemperStat::default();
            ts.update(1.7);

            let mut m: AHashMap<&[u8], TemperStat> = AHashMap::new();
            m.insert(str.as_bytes(), ts);

            (m, 1_usize, 0_usize)
        });
        let r = h.join();
        println!("{r:?}");
    });
}

#[test]
fn test_threads_spawn_join() {
    use ahash::AHashMap;
    use one_brc::TemperStat;

    const STR: &str = "hello join";

    fn test_parse_block(str: &str, v: f32) -> (AHashMap<&[u8], TemperStat>, usize, usize) {
        let mut m: AHashMap<&[u8], TemperStat> = AHashMap::new();
        let t = TemperStat::from_f32(v);
        m.insert(str.as_bytes(), t);
        (m, 1, 0)
    }

    thread::scope(|s| {
        let mut threads= Vec::new();

        for n in 369..400 {
            let h = s.spawn(move || {
                test_parse_block(STR, n as f32 / 10.0)
            });
            threads.push((h, Instant::now()));
        }

        threads.into_iter().for_each(|(h, s)| {
            let r = h.join();
            println!("{r:?}\t{:?}", s.elapsed());
        });
    });
}