use std::collections::BTreeMap;

use super::index::{Vector, generate_index};
use instant_distance::{HnswMap, Search};

pub struct Database {
    inner: HnswMap<Vector, u64>,
    pub storage: BTreeMap<u64, String>,

    pub keys: Vec<Vector>,
    pub values: Vec<u64>,
}

impl Database {
    pub fn new(keys: Vec<Vector>, values: Vec<String>) -> Self {
        let inner_values: Vec<u64> = (0..values.len() as u64).collect();
        let mut storage: BTreeMap<u64, String> = BTreeMap::new();

        for i in inner_values.clone() {
            if let Some(v) = values.get(i as usize) {
                storage.insert(i, v.clone());
            }
        }

        Database { 
            keys: keys.clone(), 
            values: inner_values.clone(), 
            inner: generate_index(keys, inner_values),
            storage,
        }
    }

    pub fn query(&self, key: &Vector, search: &mut Search, limit: i32) -> Vec<(f64, String)> {
        let mut res: Vec<(f64, String)> = vec![];
        let mut iter = self.inner.search(key, search);
        for _ in 0..limit {
            match iter.next() {
                Some(v) => {
                    if let Some(v_content) = self.storage.get(v.value) {
                        res.push((v.point.cos_sim(key), v_content.clone()))
                    }
                },
                None => break
            }
        };

        res
    }

    pub fn append(&mut self, keys: &mut Vec<Vector>, values: &mut Vec<u64>) -> Result<(), String> {
        if keys.len() != values.len() {
            return Err(String::from("length of keys not euqal to values'"));
        }
        self.keys.append(keys);
        self.values.append(values);

        Ok(())
    }

    pub fn store_one_document(&mut self, doc: String) -> Result<u64, String> {
        match self.storage.insert(self.storage.len() as u64, doc) {
            Some(_) => Ok((self.storage.len() - 1) as u64),
            None => Err(String::from("Error inserting new document")) 
        }
    }

    // reconstruct index, waiting instance-distance repo to be able support insert or we can open pr to improve
    pub fn build_index(&mut self) {
        self.inner = generate_index(self.keys.clone(), self.values.clone())
    }
}