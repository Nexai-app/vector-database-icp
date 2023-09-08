use super::index::{Vector, generate_index};
use instant_distance::{HnswMap, Search, Point};

pub struct Database {
    inner: HnswMap<Vector, String>,

    keys: Vec<Vector>,
    values: Vec<String>,
}

impl Database {
    pub fn new(keys: Vec<Vector>, values: Vec<String>) -> Self {
        Database { 
            keys: keys.clone(), 
            values: values.clone(), 
            inner: generate_index(keys, values),
        }
    }

    pub fn query(&self, key: &Vector, search: &mut Search, limit: i32) -> Vec<(f32, String)> {
        let mut res: Vec<(f32, String)> = vec![];
        let mut iter = self.inner.search(key, search);
        for _ in 0..limit {
            match iter.next() {
                Some(v) => {
                    res.push((v.distance, (*v.value).clone()))
                },
                None => break
            }
        };

        res
    }

    pub fn append(&mut self, keys: &mut Vec<Vector>, values: &mut Vec<String>) -> Result<(), String> {
        if keys.len() != values.len() {
            return Err(String::from("length of keys not euqal to values'"));
        }
        self.keys.append(keys);
        self.values.append(values);

        Ok(())
    }

    // reconstruct index, waiting instance-distance repo to be able support insert or we can open pr to improve
    pub fn build_index(&mut self) {
        self.inner = generate_index(self.keys.clone(), self.values.clone())
    }
}