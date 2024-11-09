#![feature(test)]

extern crate test;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::BuildHasher;
use std::io::Read;
use std::str::{from_utf8};
use test::Bencher;
use ahash::{AHashMap, AHashSet};

#[test]
fn test_read_stations() {
    let data = read_stations_file();
    let stations = get_stations(&data);
    println!("stations\t{}", stations.len());
    for station in stations {
        println!("{:?}", from_utf8(station).unwrap());
    }
}

fn get_stations(data: &str) -> Vec<&[u8]> {
    let mut result = Vec::new();
    let stations = data.split(|ch| ch == '\n');
    for station in stations {
        if station.len() > 0 {
            result.push(station.as_bytes());
        }
    }
    result
}

fn read_stations_file() -> String {
    let mut file = File::open("./stations.txt").unwrap();
    let len = file.metadata().unwrap().len();
    println!("file len:\t{len}");
    let mut buf = String::new();
    let read = file.read_to_string(&mut buf).unwrap();
    println!("bytes read:\t{read}");
    buf
}

#[test]
fn test_hash_collisions() {
    let data = read_stations_file();
    let stations = get_stations(&data);

    let mut set = HashSet::new();

    for station in stations {
        let hasher = set.hasher();
        let hash = hasher.hash_one(station);
        set.insert(hash);
    }
    println!("set len: {}", set.len());
}

#[test]
fn test_ahash_collisions() {
    let data = read_stations_file();
    let stations = get_stations(&data);

    let mut aset = AHashSet::new();

    for station in stations {
        let hasher = aset.hasher();
        let hash = hasher.hash_one(station);
        aset.insert(hash);
    }
    println!("aset len: {}", aset.len());
}

fn stations_hash(stations: &Vec<&[u8]>) -> u64 {
    let map:HashMap<&[u8],u64> = HashMap::new();
    let hasher = map.hasher();
    let mut res: u64 = 0;
    for station in stations {
        let h = hasher.hash_one(station);
        res &= h;
    }

    res
}

#[bench]
fn bench_stations_hash(b: &mut Bencher) {
    let data = read_stations_file();
    let stations = get_stations(&data);

    b.iter(|| stations_hash(&stations));
}

fn stations_ahash(stations: &Vec<&[u8]>) -> u64 {
    let map: AHashMap<&[u8],u64> = AHashMap::new();
    let hasher = map.hasher();
    let mut res: u64 = 0;
    for station in stations {
        let h = hasher.hash_one(station);
        res &= h;
    }

    res
}

#[bench]
fn bench_stations_ahash(b: &mut Bencher) {
    let data = read_stations_file();
    let stations = get_stations(&data);

    b.iter(|| stations_ahash(&stations));
}
