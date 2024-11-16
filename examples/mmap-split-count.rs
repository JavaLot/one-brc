use std::path::Path;
use std::time::Instant;
use one_brc::FILE_PATH;

fn main() {

    use std::fs::File;

    use memmap::Mmap;

    let start = Instant::now();
    let file = File::open(Path::new(FILE_PATH)).unwrap();

    let mmap = unsafe { Mmap::map(&file).unwrap()  };

    println!("{}", mmap.len());

    let lines = mmap.split_inclusive(|&pos| pos == b'\n').count();

    println!("counted: {lines}, elapsed: {:?}", start.elapsed());
}
