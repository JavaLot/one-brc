use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::from_utf8;
use ahash::AHashMap;
use crate::TemperStat;

/// **Temper**ature **Stat**istic **Result**
#[derive(Debug)]
pub struct TemperStatResult<'a> {
    r : BTreeMap<&'a[u8], TemperStat>
}

impl<'a> TemperStatResult<'a> {
    pub fn new() -> Self {
        TemperStatResult { r: BTreeMap::new() }
    }

    pub fn aggregate(&mut self, m: &AHashMap<&'a [u8], TemperStat>) {
        m.iter().for_each(|(&s, t)| {
            if let Some(v) = self.r.get_mut(s) {
                v.merge(t);
            } else {
                self.r.insert(s, t.clone());
            }
        })
    }
}

impl Display for TemperStatResult<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (c, (&k, v)) in self.r.iter().enumerate() {
            if let Ok(a) = from_utf8(k) {
                if c != 0 { write!(f, ", ")?; }
                write!(f, "{a}={v}")?;
            };
        };
        write!(f, "}}")
    }
}

#[test]
fn test() {
    use crate::test::TEST_STR_100;
    use crate::{process_block};

    let mut r = TemperStatResult::new();
    let (m, l, e) = process_block(TEST_STR_100.as_bytes(), 100);
    r.aggregate(&m);
    println!("{l}\t{e}\n{r}");
}