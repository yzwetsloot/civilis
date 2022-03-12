use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};

#[derive(Clone)]
pub struct Vertex {
    domain: String,
    incoming: Vec<Weak<Mutex<Vertex>>>,
    outgoing: Vec<Arc<Mutex<Vertex>>>,
}

impl Vertex {
    pub fn new(domain: String) -> Vertex {
        Vertex {
            domain,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    pub fn add_outgoing(&mut self, v: Arc<Mutex<Vertex>>) {
        self.outgoing.push(v);
    }

    pub fn add_incoming(&mut self, v: Arc<Mutex<Vertex>>) {
        self.incoming.push(Arc::downgrade(&v)); // TODO
    }
}

#[derive(Clone)]
pub struct Graph {
    vertices: Arc<Vec<Mutex<HashMap<String, Arc<Mutex<Vertex>>>>>>,
}

impl Graph {
    pub fn new(num_shards: u64) -> Graph {
        let mut sharded_graph = Vec::with_capacity(num_shards as usize);
        for _ in 0..num_shards {
            sharded_graph.push(Mutex::new(HashMap::new()));
        }

        Graph {
            vertices: Arc::new(sharded_graph),
        }
    }

    fn get_index(&self, val: &String) -> usize {
        let mut s = DefaultHasher::new();
        val.hash(&mut s);
        s.finish() as usize % self.vertices.len()
    }

    pub fn add_vertex(&self, v: Vertex) {
        let domain = v.domain.clone();
        let v = Arc::new(Mutex::new(v));

        let index = self.get_index(&domain);
        let mut vertices = self.vertices[index].lock().unwrap();

        vertices.insert(domain, v);
    }

    fn get_vertex(&self, domain: &String) -> Arc<Mutex<Vertex>> {
        let index = self.get_index(domain);
        let vertices = self.vertices[index].lock().unwrap();

        Arc::clone(&vertices.get(domain).unwrap())
    }

    pub fn add_edge(&self, src: &String, dst: &String) -> bool {
        if src == dst {
            // ignore case of self-loop or buckle
            return false;
        }
        let src = self.get_vertex(src);
        let dst = self.get_vertex(dst);

        src.lock().unwrap().add_outgoing(Arc::clone(&dst));
        dst.lock().unwrap().add_incoming(src);
        true
    }

    pub fn contains(&self, domain: &String) -> bool {
        let index = self.get_index(&domain);
        let vertices = self.vertices[index].lock().unwrap();

        vertices.contains_key(domain)
    }

    pub fn size(&self) -> usize {
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
