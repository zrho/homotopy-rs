use crate::*;

// use crate::common::Height;
use crate::graph::GraphBuilder;

use petgraph::Graph;
use std::collections::HashMap;

/// Keys are lists of slice indexes.
/// TODO: Find a better representation.
#[derive(Clone, Debug)]
pub enum Key {
    Nil,
    Cons(SliceIndex, Box<Key>),
}

pub enum Bias {
    Left,
    Right
}

/// Take a diagram and turn into a cubical graph of dimension 0.
pub fn cubicalise(diagram: &DiagramN, _bias: Vec<Bias>) -> GraphBuilder<Key> {

    let mut graph = GraphBuilder::new(Key::Nil, diagram.clone().into());

    for d in 1..diagram.dimension() + 1 {
        graph = graph
            .explode(|i, k| Key::Cons(i, Box::new(k)))
            .unwrap()
            .parallelise(d)
    }

    let x = graph.clone().build();
    log::info!("Result of cubicalisation: {:?}", x);
    graph
}

impl GraphBuilder<Key> {

    /// Take a graph and make all the nodes the same length.
    fn parallelise(self, d: usize) -> Self {
        log::info!("Parallelising in dimension {}...", d);
        let mut graph = self.clone();
        while !graph.done() {
            graph = graph
                .singular_expansion()
                .regular_expansion()
        }
        graph
    }

    /// Check if all nodes have the same length.
    fn done(&self) -> bool {
        true
    }

    fn singular_expansion(&self) -> Self {
        /// Step 1. Assign weights using topological ordering.
        let weights: HashMap<(Key, SliceIndex), usize> = HashMap::new();
        
        /// Step 2. Expand nodes using weights.
        
        /// Step 3. Expand edges using weights.
        
        /// Step 4. Reconstruct the graph.

        self.clone()
    }

    fn regular_expansion(&self) -> Self {
        self.clone()
    }

    /// Returns the topological ordering of the nodes.
    fn topological_ordering(&self) -> Vec<Vec<usize>> {
        vec![vec![]]
    }
}

impl DiagramN {

    /// Expand the singular slices according to the provided weights.
    fn expand_sing(&self, weights: usize /* TODO: What should this be? */) -> Self {
        self.clone()
    }

    /// Expand the regular slices according to the provided weights.
    fn expand_reg(&self, weights: usize /* TODO: What should this be? */) -> Self {
        self.clone()
    }
}