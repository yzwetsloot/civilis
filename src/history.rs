use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct History {
    inner: Arc<Vec<Mutex<HashSet<String>>>>,
}

impl History {
    pub fn new(num_shards: u64) -> History {
        let mut history = Vec::with_capacity(num_shards as usize);
        for _ in 0..num_shards {
            history.push(Mutex::new(HashSet::new()));
        }

        History {
            inner: Arc::new(history),
        }
    }

    fn get_index(&self, val: &String) -> usize {
        let mut s = DefaultHasher::new();
        val.hash(&mut s);
        s.finish() as usize % self.inner.len()
    }

    #[allow(dead_code)]
    pub fn contains(&self, val: String) -> bool {
        let index = self.get_index(&val);
        let set = self.inner[index].lock().unwrap();
        set.contains(&val)
    }

    pub fn insert(&self, val: String) -> bool {
        let index = self.get_index(&val);
        let mut set = self.inner[index].lock().unwrap();
        if !set.contains(&val) {
            set.insert(val);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        let mut size: usize = 0;

        for i in 0..self.inner.len() {
            let set = self.inner[i].lock().unwrap();
            size += set.len();
        }

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_empty_history(num_shards: u64) -> History {
        History::new(num_shards)
    }

    fn init_non_empty_history(num_values: usize, num_shards: u64) -> History {
        let history = History::new(num_shards);

        for i in 0..num_values {
            history.insert(format!("{} val", i));
        }

        println!("{:#?}", history);

        history
    }

    #[test]
    fn length_history_empty() {
        let history = History::new(1);
        assert_eq!(0, history.len());
    }

    #[test]
    fn length_history_non_empty() {
        let num_values = 10;
        let history = init_non_empty_history(num_values, 10);
        assert_eq!(num_values, history.len());
    }

    #[test]
    fn contains_history() {
        let history = init_empty_history(10);
        history.insert("test value".to_string());

        assert!(!history.contains("not test value".to_string()));
    }

    #[test]
    fn insert_history() {
        let history = init_empty_history(10);

        let test_value: String = "test value".to_string();

        history.insert(test_value.clone());
        assert!(history.contains(test_value.clone()));

        let not_present = history.insert(test_value);
        assert!(!not_present);

        assert_eq!(1, history.len());
    }

    #[test]
    fn insert_history_domain() {
        let history = History::new(10);

        let v = vec![
            "qunitjs.com".to_string(),
            "sizzlejs.com".to_string(),
            "webhint.io".to_string(),
            "americanexpress.com".to_string(),
            "coinbase.com".to_string(),
        ];

        for item in v {
            history.insert(item);
        }

        assert!(!history.contains("netflix.com".to_string()));
    }
}
