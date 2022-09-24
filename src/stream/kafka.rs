use anyhow::{Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CorrelationMap {
    storage: Arc<Mutex<HashMap<i32, Vec<i16>>>>
}

impl CorrelationMap {
    pub fn new() -> Self {
        CorrelationMap { 
            storage: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn add(&mut self, key: i32, value: &Vec<i16>) {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(key, value.clone());
    }

    pub fn get_and_remove(&mut self, key: i32) -> Option<Vec<i16>> {
        let mut storage = self.storage.lock().unwrap();
        let s = match storage.remove(&key) {
            Some(v) => Some(v.clone()),
            None => None,
        };
        s 
    }
}