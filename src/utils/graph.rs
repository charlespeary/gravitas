pub type Vertex<N> = (N, N);

/// (Left, Right)
/// The left vertex is the one the right connects to.
/// One node can only connect to one node on the left side.
#[derive(Default, Debug, Clone)]
pub struct Graph<N: PartialEq> {
    nodes: Vec<(N, N)>,
}

impl<N> Graph<N>
where
    N: PartialEq + Clone,
{
    pub fn add(&mut self, vertex: Vertex<N>) {
        self.nodes.push(vertex)
    }
    // Each vertex might connect only to one vertex on the left side
    pub fn get_left(&self, vertex: &N) -> Option<N> {
        self.nodes
            .iter()
            .find(|(_, right)| right == vertex)
            .map(|(l, _)| l)
            .cloned()
    }
}
