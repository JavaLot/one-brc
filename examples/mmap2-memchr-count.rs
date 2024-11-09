extern crate core;

use std::path::Path;
use std::time::Instant;
use memchr::{memchr_iter};
use one_brc::FILE_PATH;

fn main() {

    use std::fs::File;

    use memmap2::Mmap;

    let start = Instant::now();
    let file = File::open(Path::new(FILE_PATH)).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap()  };

    println!("{}", mmap.len());

    let data = &mmap[..];
    let lines= memchr_iter(b'\n', data).count();

    println!("lines: {lines}, elapsed: {:?}", start.elapsed());
}
