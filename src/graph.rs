use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};

pub struct Vertex {
    domain: String,
    incoming: Vec<Weak<Mutex<Vertex>>>,
    outgoing: Vec<Arc<Mutex<Vertex>>>,
}

impl Vertex {
    pub fn add_outgoing(&mut self, v: Arc<Mutex<Vertex>>) {
        self.outgoing.push(v);
    }
}

pub struct Graph {
    vertices: Arc<Vec<Mutex<HashMap<String, Arc<Mutex<Vertex>>>>>>,
}

impl Graph {
    pub fn new(num_shards: u64) -> Graph {
        let mut graph = Vec::with_capacity(num_shards as usize);
        for _ in 0..num_shards {
            graph.push(Mutex::new(HashMap::new()));
        }

        Graph {
            vertices: Arc::new(graph),
        }
    }

    fn get_index(&self, val: &String) -> usize {
        let mut s = DefaultHasher::new();
        val.hash(&mut s);
        s.finish() as usize % self.vertices.len()
    }

    pub fn add_vertex(&self, v: Arc<Mutex<Vertex>>) -> bool {
        let domain = v.lock().unwrap().domain.clone();

        let index = self.get_index(&domain);
        let mut vertices = self.vertices[index].lock().unwrap();

        if !vertices.contains_key(&domain) {
            vertices.insert(domain, v);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        let mut size: usize = 0;

        for i in 0..self.vertices.len() {
            let map = self.vertices[i].lock().unwrap();
            size += map.len();
        }

        size
    }

    pub fn serialize(&self) {
        todo!();
    }
}
