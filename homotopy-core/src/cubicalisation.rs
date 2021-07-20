use crate::common::*;
use crate::graph::GraphBuilder;
use crate::diagram::*;
use crate::rewrite::*;

use std::cmp::max;
use std::convert::{Into, TryFrom};

/// Keys are lists of slice indices.
/// TODO: Find a better representation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Key {
    Nil,
    Cons(Height, Box<Key>),
}

pub enum Bias {
    Left,
    Right,
}

impl Key {

    /// The degree of a key tells us where that node comes in the topological ordering.
    fn degree(&self) -> usize {
        use Height::{Regular, Singular};
        match self {
            Self::Nil => 0,
            Self::Cons(height, key) => {
                key.degree() + match height {
                    Regular(_i) => 0,
                    Singular(_i) => 1,
                }
            }
        }
    }
}

/// Take a diagram and turn into a cubical graph of dimension 0.
pub fn cubicalise(diagram: &DiagramN, _bias: Vec<Bias>) -> GraphBuilder<Key> {
    log::info!("Cubicalisation started...");
    let mut graph = GraphBuilder::new(Key::Nil, diagram.clone().into());
    graph.pretty_print();

    for d in 1..diagram.dimension() {
        log::info!("Explode in dimension {}...", d);
        graph = graph.forked_explode(|i, k| Key::Cons(i, Box::new(k))).unwrap();
        graph.pretty_print();

        log::info!("Parallelise in dimension {}...", d);
        graph = graph.parallelise(d);
        graph.pretty_print();
    }

    let d = diagram.dimension();
    log::info!("Explode in dimension {}...", d);
    graph = graph.forked_explode(|i, k| Key::Cons(i, Box::new(k))).unwrap();
    graph.pretty_print();

    graph
}

impl GraphBuilder<Key> {

    /// Take a graph and make all the nodes the same length.
    fn parallelise(&self, d: usize) -> Self {
        let mut graph = self.clone();
        graph = graph.singular_expansion(d);
        graph = graph.regular_expansion(d);
        graph
    }

    /// Check if all nodes have the same length.
    fn _done(&self) -> bool {
        true
    }

    fn singular_expansion(&self, dimension: usize) -> Self {
        log::info!("Singular expansion in dimension {}...", dimension);

        // Assign weights to singular slices.
        // weights[i][j] = weight of singular slice j of node i.
        let mut weights: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        for (degree, indices) in self.topological_ordering(dimension).iter().enumerate() {
            for i in indices.to_vec() {
                let diagram: &DiagramN = <&DiagramN>::try_from(&self.nodes[i].1).unwrap();
                // Assign weight 1 to every singular slice.
                for _ in 0..diagram.size() {
                    weights[i].push(1);
                }
                // If there are edges into this node, propagate weights forward.
                // TODO(calintat): This 'for' loop is inefficient, we can use the Key to find the edges.
                if degree > 0 {
                    for (s, t, edge) in &self.edges {
                        if *t == i {
                            let rewrite: &RewriteN = <&RewriteN>::try_from(edge).unwrap();
                            for target_height in 0..diagram.size() {
                                let weight = rewrite
                                    .singular_preimage(target_height)
                                    .map(|j| weights[*s][j])
                                    .sum();
                                weights[i][target_height] = max(weights[i][target_height], weight);
                            }
                            // for cone in rewrite.cones() {
                            //     let source_height = cone.index;
                            //     let target_height = rewrite.singular_image(source_height);
                            //     // Add the weights of the singular slices of the source cospans.
                            //     let mut sum = 0;
                            //     for j in source_height..source_height + cone.len() {
                            //         sum += weights[*s][j];
                            //     }
                            //     // The weight of the singular slice of the target cospan must be at least as big as this sum.
                            //     weights[i][target_height] = max(weights[i][target_height], sum);
                            // }
                        }
                    }
                    
                }
            }
        }
        
        // Expand nodes using weights.
        let mut expanded_nodes: Vec<(Key, Diagram)> = Vec::new();
        for i in 0..self.nodes.len() {
            let (key, node) = &self.nodes[i];
            let expanded_node: Diagram = <&DiagramN>::try_from(node)
                .unwrap()
                .singular_expansion(weights[i].clone())
                .into();
            expanded_nodes.push((key.clone(), expanded_node));
        }
        
        // Expand edges using weights.
        let mut expanded_edges: Vec<(usize, usize, Rewrite)> = Vec::new();
        for i in 0..self.edges.len() {
            let (s, t, edge) = &self.edges[i];
            let expanded_edge: Rewrite = <&RewriteN>::try_from(edge)
                .unwrap()
                .singular_expansion(weights[*s].clone(), weights[*t].clone())
                .into();
            expanded_edges.push((*s, *t, expanded_edge));
        }
        
        // Reconstruct the expanded graph.
        let graph = Self {
            nodes: expanded_nodes,
            edges: expanded_edges,
            dimension: self.dimension,
        };
        graph.pretty_print();
        graph
    }

    fn regular_expansion(&self, dimension: usize) -> Self {
        log::info!("Regular expansion in dimension {}...", dimension);

        // Assign weights to regular slices.
        // weights[i][j] = weight of regular slice j of node i.
        let mut weights: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        for (degree, indices) in self.reverse_topological_ordering(dimension).iter().enumerate() {
            for i in indices.to_vec() {
                let diagram: &DiagramN = <&DiagramN>::try_from(&self.nodes[i].1).unwrap();
                // Assign weight 1 to every singular slice.
                for _j in 0..diagram.size() + 1 {
                    weights[i].push(1);
                }
                // If there are edges out of this node, propage weights backward.
                // TODO(calintat): This 'for' loop is inefficient, we can use the Key to find the edges.
                if degree > 0 {
                    for (s, t, edge) in &self.edges {
                        if *s == i {
                            let rewrite = <&RewriteN>::try_from(edge).unwrap();
                            for target_height in 0..diagram.size() + 1 {
                                let weight = rewrite
                                    .regular_preimage(target_height)
                                    .map(|j| weights[*t][j])
                                    .sum();
                                weights[i][target_height] = max(weights[i][target_height], weight);
                            }
                        }
                    }
                }
            }
        }

        // Expand nodes using weights.
        let mut expanded_nodes: Vec<(Key, Diagram)> = Vec::new();
        for i in 0..self.nodes.len() {
            let (key, node) = &self.nodes[i];
            let expanded_node: Diagram = <&DiagramN>::try_from(node)
                .unwrap()
                .regular_expansion(weights[i].clone())
                .into();
            expanded_nodes.push((key.clone(), expanded_node));
        }
           
        // Expand edges using weights.
        let mut expanded_edges: Vec<(usize, usize, Rewrite)> = Vec::new();
        for i in 0..self.edges.len() {
            let (s, t, edge) = &self.edges[i];
            // let expanded_edge: Rewrite = <&RewriteN>::try_from(edge)
            //     .unwrap()
            // .regular_expansion(weights[*s].clone(), weights[*t].clone())
            // .into();
            expanded_edges.push((*s, *t, edge.clone() /* expanded_edge */));
        }
        
        // Reconstruct the expanded graph.
        let graph = Self {
            nodes: expanded_nodes,
            edges: expanded_edges,
            dimension: self.dimension,
        };
        graph.pretty_print();
        graph
    }

    /// Returns the topological ordering of the nodes.
    fn topological_ordering(&self, d: usize) -> Vec<Vec<usize>> {        
        let mut xss = Vec::new();
        for _i in 0..d + 1 {
            xss.push(Vec::new())
        }

        for (index, (key, _node)) in self.nodes.iter().enumerate() {
            xss[key.degree()].push(index)
        }
        
        xss
    }

    fn reverse_topological_ordering(&self, d: usize) -> Vec<Vec<usize>> {
        let mut xs = self.topological_ordering(d).clone();
        xs.reverse();
        xs
    }

    fn pretty_print(&self) {
        log::info!("Graph");
        for (i, (key, node)) in self.nodes.iter().enumerate() {
            log::info!("Node: index = {:?} key = {:?} diagram = {:?}", i, key, node);
        }
        for (s, t, edge) in &self.edges {
            log::info!("Edge: source index = {:?}, target index = {:?}, rewrite = {:?}", s, t, edge);
        }
        log::info!("\n");
    }
}

impl Cospan {

    fn identity(dimension: usize) -> Self {
        Self {
            forward: Rewrite::identity(dimension),
            backward: Rewrite::identity(dimension),
        }
    }

    fn expand(&self, n: usize) -> Vec<Self> {
        match n {
            0 => panic!("Argument must be non-zero"),
            1 => vec![self.clone()],
            _ => {
                let mut cospans = Vec::new();
                let dimension = self.forward.dimension();
                // Source cospan.
                cospans.push(Self {
                    forward: self.forward.clone(),
                    backward: Rewrite::identity(dimension),
                });
                // Intermediate cospans.
                for _ in 0..n - 2 {
                    cospans.push(Self::identity(dimension));
                }
                // Target cospan.
                cospans.push(Self {
                    forward: Rewrite::identity(dimension),
                    backward: self.backward.clone(),
                });
                cospans
            }
        }
    }
}

impl DiagramN {

    /// Expand the singular slices according to the provided weights.
    fn singular_expansion(&self, weights: Vec<usize>) -> Self {
        let mut cospans = Vec::new();
        for (i, cospan) in self.cospans().iter().enumerate() {
            cospans.extend(cospan.expand(weights[i]));
        }
        Self::new_unsafe(self.source(), cospans)
    }

    /// Expand the regular slices according to the provided weights.
    fn regular_expansion(&self, weights: Vec<usize>) -> Self {
        let mut cospans = Vec::new();
        for (i, cospan) in self.cospans().iter().enumerate() {
            for _ in 0..weights[i] - 1 {
                cospans.push(Cospan::identity(self.dimension() - 1));
            }
            cospans.push(cospan.clone());
        }
        for _ in 0..weights[self.size()] - 1 {
            cospans.push(Cospan::identity(self.dimension() - 1));
        }
        Self::new_unsafe(self.source(), cospans)
    }
}

impl RewriteN {

    /// Expand the singular slices according to the provided weights.
    fn singular_expansion(
        &self,
        source_weights: Vec<usize>,
        target_weights: Vec<usize>
    ) -> Self {
        let mut index = 0;
        let mut cones = Vec::new();
        // TOOD(calintat): This ignores identity cones, should iterate over target heights instead.
        for cone in self.cones() {
            let source: &Vec<Cospan> = &cone.internal.source;
            let target: &Cospan = &cone.internal.target;
            let slices: &Vec<Rewrite> = &cone.internal.slices;

            let source_height = cone.index;
            let target_height = self.singular_image(source_height);
            let mut expanded_target = target.expand(target_weights[target_height]);

            for (i, slice) in slices.iter().enumerate() {
                for x in source[i].expand(source_weights[source_height + i]) {
                    cones.push(Cone::new(
                        index,
                        vec![x],
                        expanded_target.remove(0),
                        vec![slice.clone()],
                    ));
                    index += 1;
                }
            }
            for x in expanded_target {
                cones.push(Cone::new(
                    index,
                    vec![],
                    x,
                    vec![],
                ))
            }
        }
        Self::new(self.dimension(), cones)
    }

    /// Expand the regular slices according to the provided weights.
    fn regular_expansion(
        &self,
        source_weights: Vec<usize>,
        target_weights: Vec<usize>
    ) -> Self {
        self.clone()
    }
}