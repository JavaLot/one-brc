pub mod result;

use std::fmt::{Debug, Display, Formatter};
use ahash::AHashMap;
use memchr::{memchr_iter, memrchr};

pub const STATION_NAME_MAX_LEN: usize = 100;
pub const MEASSUREMENT_MAX_LEN: usize = 5;
pub const LINE_MAX_LEN: usize = STATION_NAME_MAX_LEN + MEASSUREMENT_MAX_LEN + 2; // STATION_NAME_MAX_LEN;MEASSUREMENT_MAX_LEN\n

#[cfg(debug_assertions)]
pub const FILE_PATH: &str = "measurements-small.txt"; // Debug configuration
#[cfg(not(debug_assertions))]
pub const FILE_PATH: &str = "measurements.txt"; // Release configuration

/// Parse temperature byte slice to Option<i16> value
/// Return None if parse error
pub fn i16_from_bytes(buf: &[u8]) -> Option<i16> {
    let buf_len = buf.len();

    if buf_len > 2 && buf_len < 6 && buf[buf_len - 2] == b'.' {

        let mut fractional = buf[buf_len - 1];
        match fractional {
            b'0'..=b'9' => fractional = fractional - b'0',
            _ => return None
        }

        let integral_last_pos = buf_len - 3;
        let mut integral_last = buf[integral_last_pos];
        match integral_last {
            b'0'..=b'9' => integral_last = integral_last - b'0',
            _ => return None
        }

        let negative = buf[0] == b'-';
        let integral_first_pos: usize = if negative { 1 } else { 0 };

        return match integral_last_pos - integral_first_pos {
            0 => {
                if negative {
                    Some(-((integral_last as i16 * 10) + (fractional as i16)))
                } else {
                    Some((integral_last as i16 * 10) + (fractional as i16))
                }
            }
            1 => {
                let mut integral_first = buf[integral_first_pos];
                match integral_first {
                    b'0'..=b'9' => integral_first = integral_first - b'0',
                    _ => return None
                }

                if negative {
                    Some(-((integral_first as i16 * 100) + (integral_last as i16 * 10) + fractional as i16))
                } else {
                    Some((integral_first as i16 * 100) + (integral_last as i16 * 10) + fractional as i16)
                }
            }
            _ => { None }
        }
    }

    None
}

#[test]
fn test_i16_from_bytes() {
    // all good numbers
    for i in -999..=999 {
        let f = (i as f32) / 10.0;
        let string = format!("{f:.1}");
        let b = string.as_bytes();
        let p = crate::i16_from_bytes(b).unwrap();
        assert_eq!(i, p);
    }

    // some bad numbers
    assert_eq!(None, crate::i16_from_bytes("100.1".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("1000.1".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes(".1".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes(".11".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("100".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("1c.0".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("1 .0".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("10.b".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("10. ".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes("+0.0".as_bytes()));
    assert_eq!(None, crate::i16_from_bytes(" 1.0".as_bytes()));
}

/// **Temper**ature **Stat**istics
#[derive(Clone, Debug)]
pub struct TemperStat {
    min: i16,
    max: i16,
    sum: i64,
    count: usize
}

impl TemperStat {
    pub fn from_i16(v: i16) -> Self {
        TemperStat {
            min: v,
            max: v,
            sum: v as i64,
            count: 1
        }
    }

    pub fn update(&mut self, v: i16) {
        if self.min > v {
            self.min = v;
        }
        if v > self.max {
            self.max = v;
        }
        self.sum += v as i64;
        self.count += 1;
    }

    pub fn merge(&mut self, other: &Self) {
        if self.min > other.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max
        }
        self.sum += other.sum;
        self.count += other.count;
    }
}

impl Display for TemperStat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}/{:.1}/{:.1}", self.min as f32 / 10.0, self.sum as f64 / (self.count as f64 * 10.0), self.max as f32 / 10.0)
    }
}

#[test]
fn test_temper_stat_i16() {
    let mut a = TemperStat::from_i16(19);
    println!("after from_i16 19: {a}");
    dbg!(&a);
    a.update(998);
    println!("after update_i16 998: {a}");
    dbg!(&a);
    a.update(-105);
    println!("after update_i16 -105: {a}");
    dbg!(&a);

    a.merge(&a.clone());
    println!("after merge with self.clone: {a}");
    dbg!(&a);
}

/// The rounding must be done using the semantics of IEEE 754 rounding-direction "roundTowardPositive",
/// one digit after decimal point.
pub fn temp_round(x: f64) -> f32 {
    let v = x * 10.0;
    let mut r = v.trunc();
    if v > r { r += 1.0; }
    (r / 10.0) as f32
}

/// Parse block of lines to map of stations name and temperature statistics, parsed line counter, errors counter
pub fn process_block(block: &[u8], capacity: usize) -> (AHashMap<&[u8],TemperStat>, usize, usize) {
    let mut map: AHashMap<&[u8],TemperStat> = AHashMap::with_capacity(capacity);

    let mut line_count: usize = 0;
    let mut error_count: usize = 0;

    let mut begin: usize = 0;

    let mut m = memchr_iter(b'\n', block);

    while let Some(end) = m.next() {
        let line = &block[begin..end];

        line_count += 1;
        begin = end + 1;

        if let Some(col) = memrchr(b';', line) {
            let (name, temp) = (&line[0..col], &line[(col + 1)..]);
            if let Some(t) = i16_from_bytes(temp) {
                if let Some(c) = map.get_mut(name) {
                    c.update(t);
                } else {
                    map.insert(name, TemperStat::from_i16(t));
                }
            } else {
                // let str = from_utf8(temp).unwrap_or("$$$$");
                // eprintln!("Incorrect temperature `{}` in line {line_count}", str);
                error_count += 1;
                continue
            }
        } else {
            // eprintln!("No column delimiter in line {line_count}");
            error_count += 1;
            continue
        }
    }

    (map, line_count, error_count)
}

#[test]
fn test_process_block() {
    let block = test::TEST_STR_100.as_bytes();
    let (map, lines, errors) = process_block(block, 100);
    assert_eq!(90, map.len());
    assert_eq!(100, lines);
    assert_eq!(0, errors);
}

pub mod test {

    pub const TEST_STR_1: &str = "\
愛媛県今治市;20.8
";

    pub const TEST_STR_100: &str = "\
愛媛県今治市;20.8
Brussels;14.9
Lake Tekapo;6.6
Astana;-14.2
Москва;-13.7
San Diego;22.3
Kinshasa;32.1
Cape Town;26.7
Colombo;8.8
Bangkok;31.5
Mango;43.9
New Delhi;23.2
Hiroshima;10.4
Xi'an;5.5
Assab;21.5
Jayapura;24.7
London;29.8
Lusaka;29.6
Anadyr;-33.3
Harare;22.7
London;1.3
Cape Town;28.3
Johannesburg;11.7
Novosibirsk;19.5
Alexandra;16.5
Barcelona;18.2
Las Palmas de Gran Canaria;11.4
Andorra la Vella;21.6
Marseille;-2.6
Nairobi;17.4
Kuopio;0.2
Bujumbura;19.9
Vancouver;3.9
Bata;41.3
Kunming;6.7
Tamale;30.1
Reggane;33.2
Roseau;20.0
Maun;27.4
Maputo;44.7
Austin;12.8
Baku;20.8
Kolkata;28.7
Brisbane;16.5
Rome;15.2
Berlin;-7.0
Bissau;31.4
Kandi;29.9
Dar es Salaam;23.9
Dili;34.4
St. Louis;12.7
Taipei;3.0
Seattle;17.5
Timbuktu;19.2
Kyiv;6.2
Bergen;14.9
Zagreb;11.4
Salt Lake City;12.8
Kingston;45.3
Thiès;17.1
Oklahoma City;-7.4
Petropavlovsk-Kamchatsky;5.4
Manama;41.3
San Francisco;31.9
Dikson;-13.4
Napier;11.2
Murmansk;-13.6
Tamale;33.4
Podgorica;16.7
Yerevan;-2.6
Napoli;6.1
Lake Havasu City;23.8
Parakou;24.9
Maputo;31.2
Perth;34.7
Monaco;7.9
Manama;17.6
Nakhon Ratchasima;37.1
Zagreb;11.3
Tokyo;-11.5
Stockholm;9.8
Sydney;17.4
Kunming;-4.2
San Antonio;7.4
Bratislava;2.4
Yakutsk;-0.6
Palm Springs;25.7
Erbil;32.1
Cotonou;17.2
Nashville;5.5
Lagos;25.4
Abidjan;17.9
Johannesburg;19.8
Kinshasa;39.9
Gangtok;25.4
Tauranga;1.6
Salt Lake City;28.5
Khartoum;34.4
Livingstone;42.2
Phnom Penh;32.9
";

}