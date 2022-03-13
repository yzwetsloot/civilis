use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::{fmt, io};

type SyncVertex = Arc<Mutex<Vertex>>;

#[derive(Clone)]
pub struct Vertex {
    domain: String,
    incoming: Vec<Weak<Mutex<Vertex>>>,
    outgoing: Vec<SyncVertex>,
}

impl Vertex {
    pub fn new(domain: String) -> Vertex {
        Vertex {
            domain,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    pub fn add_outgoing(&mut self, v: SyncVertex) -> bool {
        // TODO: change parameter type to Vertex
        if self.domain == v.lock().unwrap().domain {
            // prevent self-loop
            return false;
        }
        self.outgoing.push(v);
        true
    }

    pub fn add_incoming(&mut self, v: SyncVertex) -> bool {
        // TODO: change parameter type to Vertex instead of SyncVertex
        if self.domain == v.lock().unwrap().domain {
            // prevent self-loop
            return false;
        }
        self.incoming.push(Arc::downgrade(&v)); // TODO: verify correct
        true
    }

    pub fn in_degree(&self) -> usize {
        self.incoming.len()
    }

    pub fn out_degree(&self) -> usize {
        self.outgoing.len()
    }

    pub fn serialize() {
        todo!();
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (in {}, out {})",
            self.domain,
            self.in_degree(),
            self.out_degree()
        )
    }
}

type SyncShardedHashMap<T, V> = Arc<Vec<Mutex<HashMap<T, V>>>>;

#[derive(Clone)]
pub struct Graph {
    vertices: SyncShardedHashMap<String, SyncVertex>,
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

    fn get_shard(&self, domain: &String) -> MutexGuard<HashMap<String, SyncVertex>> {
        let index = self.get_index(&domain);
        self.vertices[index].lock().unwrap()
    }

    pub fn add_vertex(&self, v: Vertex) {
        let domain = v.domain.clone();
        let v = Arc::new(Mutex::new(v));

        let mut vertices = self.get_shard(&domain);
        vertices.insert(domain, v);
    }

    fn get_vertex(&self, domain: &String) -> Option<SyncVertex> {
        let vertices = self.get_shard(domain);
        let vertex = vertices.get(domain)?;

        Some(Arc::clone(&vertex))
    }

    pub fn add_edge(&self, src: &String, dst: &String) -> Result<(), &str> {
        if src == dst {
            // ignore case of self-loop or buckle
            return Err("source and destination vertex are the same");
        }
        let src = self.get_vertex(src).ok_or("missing source vertex")?;
        let dst = self.get_vertex(dst).ok_or("missing destination vertex")?;

        src.lock().unwrap().add_outgoing(Arc::clone(&dst));
        dst.lock().unwrap().add_incoming(src);
        Ok(())
    }

    pub fn contains(&self, domain: &String) -> bool {
        let vertices = self.get_shard(domain);
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

    pub fn serialize(&self) -> io::Result<()> {
        use std::fs;

        let mut text = String::new();

        for shard in self.vertices.iter() {
            let m = shard.lock().unwrap();
            for v in m.values() {
                let v = v.lock().unwrap();
                text += format!("{}\n", v).as_str();
            }
        }

        fs::write("out/graph", text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_vertex() {
        // mimics: github.com -> facebook.com -> stackoverflow.com
        let src = Vertex::new("github.com".to_string());
        let mut mid = Vertex::new("facebook.com".to_string());
        let dst = Vertex::new("stackoverflow.com".to_string());

        mid.add_incoming(Arc::new(Mutex::new(src)));
        mid.add_outgoing(Arc::new(Mutex::new(dst)));

        assert_eq!(format!("{}", mid), "facebook.com (in 1, out 1)");
    }

    #[test]
    fn add_outgoing_self_loop_vertex() {
        let mut vertex = Vertex::new("github.com".to_string());

        let cloned_vertex = Arc::new(Mutex::new(vertex.clone()));
        let was_added = vertex.add_outgoing(cloned_vertex);

        assert!(!was_added);

        assert_eq!(vertex.out_degree(), 0);
    }

    #[test]
    fn add_outgoing_vertex() {
        let mut src = Vertex::new("github.com".to_string());
        let dst = Vertex::new("stackoverflow.com".to_string());

        src.add_outgoing(Arc::new(Mutex::new(dst)));

        assert_eq!(src.out_degree(), 1);
    }

    #[test]
    fn add_incoming_self_loop_vertex() {
        let mut vertex = Vertex::new("github.com".to_string());

        let cloned_vertex = Arc::new(Mutex::new(vertex.clone()));
        let was_added = vertex.add_incoming(cloned_vertex);

        assert!(!was_added);

        assert_eq!(vertex.in_degree(), 0);
    }

    #[test]
    fn add_incoming_vertex() {
        let mut src = Vertex::new("github.com".to_string());
        let dst = Vertex::new("stackoverflow.com".to_string());

        src.add_incoming(Arc::new(Mutex::new(dst)));

        assert_eq!(src.in_degree(), 1);
    }

    fn init_empty_graph(num_shards: u64) -> Graph {
        Graph::new(num_shards)
    }

    fn init_non_empty_graph(num_values: usize, num_shards: u64) -> Graph {
        let graph = Graph::new(num_shards);

        for i in 0..num_values {
            graph.add_vertex(Vertex::new(format!("{} val", i)));
        }

        graph
    }

    #[test]
    fn size_graph_empty() {
        let graph = init_empty_graph(1);
        assert_eq!(0, graph.size());
    }

    #[test]
    fn size_graph_non_empty() {
        let num_values = 10;
        let graph = init_non_empty_graph(num_values, 1);
        assert_eq!(num_values, graph.size());
    }

    #[test]
    fn contains_graph() {
        let graph = init_empty_graph(10);
        assert!(!graph.contains(&String::from("github.com")));
    }

    #[test]
    fn add_vertex_graph() {
        let graph = init_empty_graph(10);

        let domain = String::from("github.com");

        graph.add_vertex(Vertex::new(domain.clone()));
        assert!(graph.contains(&domain));
    }

    #[test]
    fn add_edge_graph() {
        let graph = init_empty_graph(10);

        let src = String::from("github.com");
        let dst = String::from("google.com");

        graph.add_vertex(Vertex::new(src.clone()));
        graph.add_vertex(Vertex::new(dst.clone()));

        graph.add_edge(&src, &dst).unwrap(); // if Err -> test FAIL

        graph.serialize().unwrap();
    }

    #[test]
    #[should_panic]
    fn add_edge_missing_vertex_graph() {
        let graph = init_empty_graph(10);

        let src = String::from("github.com");
        let dst = String::from("stackoverflow.com");

        graph.add_vertex(Vertex::new(src.clone()));

        graph.add_edge(&src, &dst).unwrap(); // will panic!
    }
}
