use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;
use rand::Rng;
use one_brc::STATION_NAME_MAX_LEN;

const REQUIRED_KEYS: usize = 10_000;
const LINES_IN_TARGET: usize = 1_000_000_000;
const TARGET_FILE_NAME: &str = "measurements.txt";
const STATION_PATH: &str = "stations2.txt";

const BUFFER_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug)]
struct WeatherStation {
    name: String,
    temp: f32
}

impl WeatherStation {
    fn new(name: String, temp: f32) -> Self {
        Self {
            name,
            temp
        }
    }
}

fn main() {
    let start = Instant::now();

    let (stations, len, lines, bytes) = read_stations();

    verify(&stations);
    info(&stations);

    println!("Unique keys: {}", stations.len());
    println!("{:?}", start.elapsed());
    println!("file length: {len}, lines: {lines}, bytes: {bytes}");

    println!("-->\tStart generate keys");
    let start_required_map = Instant::now();

    let mut required_map: HashMap<&String, f32> = HashMap::with_capacity(REQUIRED_KEYS);
    stations.iter().for_each(|s| {required_map.insert(&s.name, s.temp);});

    let mut generated_map:HashMap<String, f32> = HashMap::with_capacity(REQUIRED_KEYS);

    while (required_map.len() + generated_map.len()) < REQUIRED_KEYS {
        let new = generate_key(&stations, &required_map, &generated_map);
        match new {
            Some((n, t)) => {generated_map.insert(n, t);}
            None => {}
        }
    }
    for (n, t) in generated_map.iter() {
        required_map.insert(n, *t);
    }
    println!("\trequired map capacity: {}, len: {}", required_map.capacity(), required_map.len());
    println!("\tgenerated map capacity: {}, len: {}", generated_map.capacity(), generated_map.len());
    println!("-->\tEnd generate keys: {:?}", start_required_map.elapsed());

    println!("==>\tStart write data");
    let start_write_data = Instant::now();
    let mut c: usize = 0;
    let mut rng = rand::thread_rng();

    let mut writer = BufWriter::with_capacity(BUFFER_SIZE, File::create(TARGET_FILE_NAME).unwrap());
    println!("\tFile writer capacity: {}", writer.capacity());

    while c < LINES_IN_TARGET {
        required_map.iter().take_while(|_|  {
            c += 1;
            c < LINES_IN_TARGET + 1
        }).for_each(|(station, t)| {
            let temp: f32 = rng.gen_range((t - 20.0)..(t + 20.0));
            writeln!(writer, "{station};{temp:.1}").unwrap();
        });
    }

    match writer.flush() {
        Ok(_) => {println!("\t{TARGET_FILE_NAME} saved!")}
        Err(e) => {println!("\t{}", e)}
    }
    println!("==>\tEnd write data: {:?}", start_write_data.elapsed());
}

fn read_stations() -> (Vec<WeatherStation>, u64, usize, usize) {
    let mut stations = Vec::new();

    let file = File::open(STATION_PATH).unwrap();
    let len = file.metadata().unwrap().len();
    let reader = BufReader::new(file);

    let mut lines: usize = 0;
    let mut bytes: usize = 0;

    for line in reader.lines() {
        let l = line.unwrap();
        let v: Vec<&str> = l.split(';').collect();

        let ws: WeatherStation = {
            let n: String = v[0].to_string();
            let t: f32 = v[1].parse().unwrap();
            WeatherStation::new(n, t)
        };

        let len = l.len();

        lines += 1;
        bytes += len;

        stations.push(ws);
    }

    (stations, len, lines, bytes)
}

fn verify(stations: &Vec<WeatherStation>) {
    let mut stations_set: HashSet<&String> = HashSet::new();

    let mut unique = true;
    let mut very_long = false;

    for (i, station) in stations.iter().enumerate() {
        let len = station.name.len();

        if len > STATION_NAME_MAX_LEN {
            very_long = true;
            eprintln!("Long line: {}\t{len}\t{}\t{:?}", i + 1, station.name, station.name.as_bytes());
        }

        if !stations_set.insert(&station.name) {
            unique = false;
            eprintln!("not unique line: {}\t{}", i + 1, station.name);
        }
    }

    if !unique {
        eprintln!("Stations file contains not unique rows!");
    }
    if very_long {
        eprintln!("Stations file contains very long names!");
    }

    if !unique || very_long { panic!() }
}

fn info(stations: &Vec<WeatherStation>) {
    let mut max_len: usize = 0;
    let mut max_index: usize = 0;
    for (i, station) in stations.iter().enumerate() {
        let len = station.name.len();
        if len > max_len {
            max_len = len;
            max_index = i;
        }
    }
    println!("max len: {max_len}, line: {}\t{}", max_index + 1, stations.get(max_index).unwrap().name);
}

fn generate_key(stations: &Vec<WeatherStation>, required_map: &HashMap<&String, f32>, generated_map: &HashMap<String, f32>) -> Option<(String, f32)> {
    let rand: usize = rand::thread_rng().gen_range(0..stations.len());
    let station = stations.get(rand).unwrap();
    let name = &station.name;
    let temp = station.temp;
    let mut i = 0;
    let mut str = name.clone();
    while (required_map.contains_key(&str) || generated_map.contains_key(&str)) && i < 10 {
        i += 1;
        str = format!("{name}-{i}");
    }
    if i == 10 || str.len() > STATION_NAME_MAX_LEN { None } else { Some((str, temp)) }
}

#[cfg(test)]
mod test {
    use rand::Rng;
    use crate::WeatherStation;

    #[test]
    fn test_increase_string() {
        let str = &String::from("test");
        let mut i = 0;
        while i < 10 {
            let iter = &format!("{str}-{i}");
            println!("{iter}");
            i += 1;
        }
    }

    #[test]
    fn generate_rand() {
        let mut rng = rand::thread_rng();

        for _ in 0..5 {
            let temp: f32 = rng.gen_range(-99.9..100.0);
            println!("range = {:.1}", temp);
        }
    }

    const STRING: &str = "\
Zürich;53.1
Москва;-9.2
愛媛県今治市;64.1
Майкоп;38.6
bad;234234;234234
";

    #[test]
    fn test_parse_stations2_lines() {
        for (i, line) in STRING.lines().enumerate() {
            let v: Vec<&str> = line.split(';').collect();
            let ws:Option<WeatherStation> = if v.len() != 2 {
                println!("Bad line {i}\t{line}");
                None
            } else {
                let n: String = v[0].to_string();
                let t: f32 = v[1].parse().unwrap();
                Some(WeatherStation::new(n, t))
            };

            println!("{ws:?}");
        }

        let s = WeatherStation::new("test".to_string(), 3.9);
        println!("{s:?}");
    }
}
