
use std::collections::HashMap;
use std::fmt;




pub struct Timings<T: Sync + Send + 'static> {
    pub timings: HashMap<String, f64>,
    pub others: Vec<(String, T)>,
}

impl<T> Timings<T>
where
    T: Sync + Send + 'static,
{
    pub fn new() -> Timings<T> {
        Timings {
            timings: HashMap::new(),
            others: Vec::new(),
        }
    }

    pub fn add(&mut self, k: &str, v: f64) {
        self.timings.insert(String::from(k), v);
    }
    pub fn add_other(&mut self, k: &str, v: T) {
        self.others.push((String::from(k), v));
    }

    pub fn combine(&mut self, mut other: Self) {
        for (k, v) in other.timings {
            if self.timings.contains_key(&k) {
                *self.timings.get_mut(&k).unwrap() += v;
            } else {
                self.timings.insert(k, v);
            }
        }
        for (a, b) in std::mem::take(&mut other.others) {
            self.others.push((a, b));
        }
    }
}
impl<T> fmt::Display for Timings<T>
where
    T: Sync + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fs = String::new();
        for (k, v) in &self.timings {
            fs = format!("{}\n{}: {:0.1}s", fs, k, v);
        }
        write!(f, "Timings: {}", fs)
    }
}
