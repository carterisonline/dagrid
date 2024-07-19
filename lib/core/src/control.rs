use std::borrow::Cow;

use petgraph::algo::DfsSpace;
use petgraph::csr::IndexType;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::{NodeIndices, StableDiGraph};
use petgraph::visit::{EdgeRef, Visitable};
use petgraph::{Direction, Incoming, Outgoing};
use serde::{Deserialize, Serialize};

use crate::container::Container;
use crate::node::*;
use crate::Sample;

pub struct Neighbor {
    pub node_index: NodeIndex,
    pub edge_index: EdgeIndex,
    pub dest_port_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeData {
    input_arena_ptr: usize,
    #[serde(skip)]
    gen: u64,
    #[serde(skip)]
    val: Sample,
    node: Box<dyn Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlGraph {
    #[serde(skip)]
    phase: u64,
    sample_rate: u32,
    pub dag: StableDiGraph<NodeData, usize, u32>,
    #[serde(skip)]
    dag_cycle_state: DfsSpace<NodeIndex, <StableDiGraph<NodeData, usize, u32> as Visitable>::Map>,
    node_input_arena: Vec<NodeIndex>,
    node_input_val_arena: Vec<Sample>,
    container_idents: Vec<String>,
    container_stack: Vec<usize>,
    container_members: Vec<Vec<NodeIndex>>,
    container_children: Vec<Vec<usize>>,
    aout_node: NodeIndex,
    #[serde(skip)]
    cache: Vec<(NodeIndex, usize)>,
    #[serde(skip)]
    cache_invalid: bool,
}

impl ControlGraph {
    /// Returns a new control graph with its `sample_rate` set.
    pub fn new(sample_rate: u32) -> Self {
        let mut dag = StableDiGraph::new();
        let aout_node = dag.add_node(NodeData {
            input_arena_ptr: 0,
            gen: 0,
            val: Sample::default(),
            node: Box::new(Empty),
        });
        Self {
            phase: 0,
            sample_rate,
            dag,
            dag_cycle_state: DfsSpace::default(),
            node_input_arena: vec![0.into()],
            node_input_val_arena: vec![f64::NAN.into()],
            container_idents: vec![],
            container_stack: vec![],
            container_members: vec![],
            container_children: vec![vec![]],
            aout_node,
            cache: vec![],
            cache_invalid: true,
        }
    }

    /// Loads a control graph from the postcard-encoded data.
    pub fn load(sample_rate: u32, data: &[u8]) -> postcard::Result<Self> {
        let mut cg: Self = postcard::from_bytes(data)?;

        cg.phase = 0;
        cg.sample_rate = sample_rate;
        cg.cache_invalid = true;

        Ok(cg)
    }

    /// Saves the control graph into postcard-encoded data.
    pub fn save(&self) -> postcard::Result<Vec<u8>> {
        postcard::to_stdvec(self)
    }

    /// Inserts a node into the control graph.
    ///
    /// Returns the index of the node.
    pub fn insert<N: Node + Send + 'static>(&mut self, n: N) -> NodeIndex {
        let input_len = n.get_input_labels().len();
        let node = self.dag.add_node(NodeData {
            input_arena_ptr: self.node_input_arena.len(),
            gen: self.phase,
            val: Sample::default(),
            node: Box::new(n),
        });

        for _ in 0..input_len {
            self.node_input_arena.push(0.into());
            self.node_input_val_arena.push(f64::NAN.into());
        }

        for &i in &self.container_stack {
            self.container_members[i].push(node);
        }

        node
    }

    /// Removes a node from the control graph, severing all connections with other nodes.
    ///
    /// Returns `Some(NodeData)` if the removal was successful.
    /// Returns `None` if the node doesn't exist.
    pub fn remove(&mut self, node: NodeIndex) -> Option<NodeData> {
        self.dag.remove_node(node)
    }

    /// Disconnects an edge from the control graph.
    ///
    /// Returns `Some(usize)` if the removal was successful.
    /// Returns `None` if the edge doesn't exist.
    pub fn disconnect(&mut self, edge: EdgeIndex) -> Option<usize> {
        self.dag.remove_edge(edge)
    }

    /// Traverses the entire control graph beginning at `aout`.
    ///
    /// Returns the next sample.
    pub fn next_sample(&mut self) -> Sample {
        let sample = self.update_node(
            self.dag
                .neighbors_directed(self.aout_node, Incoming)
                .next()
                .unwrap(),
        );

        self.phase += 1;
        self.cache_invalid = false;

        sample
    }

    fn update_node(&mut self, node: NodeIndex) -> Sample {
        if self.cache_invalid {
            let mut parents = self.dag.neighbors_directed(node, Incoming).detach();
            let input_arena_ptr = self.dag.node_weight(node).unwrap().input_arena_ptr;

            while let Some((e, n)) = parents.next(&self.dag) {
                let parent_node = self.dag.node_weight(n).unwrap();
                let edge_id = *self.dag.edge_weight(e).unwrap();
                if parent_node.gen <= self.phase {
                    self.update_node(n);
                }

                self.node_input_arena[input_arena_ptr + edge_id] = n;
            }

            // Set generation to `u64::MAX` for const nodes to avoid recalculating
            if self.dag.node_weight(node).unwrap().node.get_ident() == "Constant" {
                self.dag.node_weight_mut(node).unwrap().gen = u64::MAX;
            } else {
                self.dag.node_weight_mut(node).unwrap().gen = self.phase + 1;
            }

            self.cache.push((node, input_arena_ptr));

            let inputs = update_node_inputs(
                &self.dag,
                node,
                input_arena_ptr,
                &mut self.node_input_val_arena,
                &mut self.node_input_arena,
            );

            let node = &mut self.dag.node_weight_mut(node).unwrap();

            let val = node.node.process(
                &self.node_input_val_arena[input_arena_ptr..(input_arena_ptr + inputs)],
                self.phase,
                self.sample_rate,
            );

            node.val = val;

            val
        } else {
            let mut val = Sample::default();
            for (node, input_arena_ptr) in &self.cache {
                let inputs = update_node_inputs(
                    &self.dag,
                    *node,
                    *input_arena_ptr,
                    &mut self.node_input_val_arena,
                    &mut self.node_input_arena,
                );

                let node = &mut self.dag.node_weight_mut(*node).unwrap();
                val = node.node.process(
                    &self.node_input_val_arena[*input_arena_ptr..(input_arena_ptr + inputs)],
                    self.phase,
                    self.sample_rate,
                );

                node.val = val;
            }

            val
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    /// Returns the neighbors of the specified node.
    pub fn get_node_neighbors(&self, node: NodeIndex, direction: Direction) -> Vec<Neighbor> {
        self.dag
            .edges_directed(node, direction)
            .map(|e| (e.id(), e.target()))
            .map(|(e, n)| Neighbor {
                node_index: n,
                edge_index: e,
                dest_port_id: *self.dag.edge_weight(e).unwrap(),
            })
            .collect::<Vec<_>>()
    }

    /// Returns all node indexes contained in the control graph.
    pub fn get_node_indexes(&self) -> NodeIndices<NodeData, u32> {
        self.dag.node_indices()
    }

    pub fn get_node(&self, id: NodeIndex) -> &dyn Node {
        self.dag.node_weight(id).unwrap().node.as_ref()
    }

    fn push_container_layer(&mut self) {
        self.container_members.push(vec![]);
        self.container_children.push(vec![]);
        self.container_stack.push(self.container_members.len() - 1)
    }

    fn pop_container_layer(&mut self) -> Option<usize> {
        self.container_stack.pop()
    }

    pub fn get_container_children(&self) -> &Vec<Vec<usize>> {
        &self.container_children
    }

    pub fn get_container_member_indexes(&self, i: usize) -> std::slice::Iter<NodeIndex> {
        self.container_members[i].iter()
    }

    pub fn get_container_ident(&self, i: usize) -> &str {
        &self.container_idents[i]
    }

    pub fn insert_container<C: Container>(
        &mut self,
        container: C,
    ) -> (Vec<NodeIndex>, Vec<NodeIndex>) {
        let parent = self
            .container_stack
            .last()
            .map(|i| i + 1)
            .unwrap_or_default();

        self.container_children[parent].push(self.container_members.len());

        self.container_idents.push(container.get_ident().into());

        self.push_container_layer();

        let input_labels = container.get_input_labels();
        let mut inputs = Vec::with_capacity(input_labels.len());
        for l in input_labels {
            inputs.push(self.insert(ContainerInput([Cow::Owned(l.to_string())])));
        }

        let output_labels = container.get_output_labels();
        let mut outputs = Vec::with_capacity(output_labels.len());
        for l in output_labels {
            outputs.push(self.insert(ContainerOutput([Cow::Owned(l.to_string())])));
        }

        container.construct(&inputs, &outputs, self);

        self.pop_container_layer();

        (inputs, outputs)
    }
}

// WARNING: INCREDIBLE POLYFILL BS AHEAD. THIS SUCKS A LOT
// I tried making a single `connect` function using enums to modify the behavior but it was way worse to use
impl ControlGraph {
    /// Connects an existing node (`src`) into another existing node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    pub fn connect_ex_ex(&mut self, src: NodeIndex, dest: NodeIndex) {
        self.connect_ex_ex_port(src, dest, 0);
    }

    /// Connects an existing node (`src`) into another existing node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    pub fn connect_ex_ex_port(&mut self, src: NodeIndex, dest: NodeIndex, dest_port: usize) {
        if would_cycle(&self.dag, src, dest, &mut self.dag_cycle_state) {
            panic!(
                "Adding edge {src:?} -> {dest:?} would cause a cycle!\n\nGraph:{:?}",
                self.dag
            );
        }

        self.cache_invalid = true;
        self.dag.add_edge(src, dest, dest_port);
    }
    /// Connects an existing node (`src`) into another existing node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    /// Alias to [ControlGraph::connect_existing_existing_port].
    pub fn connect(&mut self, src: NodeIndex, dest: NodeIndex, dest_port: usize) {
        self.connect_ex_ex_port(src, dest, dest_port)
    }

    /// Connects a node to `aout`, which represents the final node in the graph.
    pub fn connect_existing_aout(&mut self, a: NodeIndex) {
        self.connect_ex_ex_port(a, self.aout_node, 0);
    }

    /// Connects many existing nodes (`srcs`) into another existing node (`dest`).
    /// `srcs[0]` will connect to port 0 of `dest`, `srcs[1]` will connect to port 1, etc.
    pub fn connect_many_ex(&mut self, srcs: &[NodeIndex], dest: NodeIndex) {
        for (i, src) in srcs.iter().enumerate() {
            self.connect_ex_ex_port(*src, dest, i);
        }
    }

    /// Connects many existing nodes (`srcs`) into a new node (`dest`).
    /// `srcs[0]` will connect to port 0 of `dest`, `srcs[1]` will connect to port 1, etc.
    ///
    /// Returns the index of `dest`.
    pub fn connect_many_new<N: Node + Send + 'static>(
        &mut self,
        srcs: &[NodeIndex],
        dest: N,
    ) -> NodeIndex {
        let dest_index = self.insert(dest);

        for (i, src) in srcs.iter().enumerate() {
            self.connect_ex_ex_port(*src, dest_index, i);
        }

        dest_index
    }

    /// Connects an existing node (`src`) into a new node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    ///
    /// Returns the index of `dest`.
    pub fn connect_ex_new<N: Node + Send + 'static>(
        &mut self,
        src: NodeIndex,
        dest: N,
    ) -> NodeIndex {
        let dest_index = self.insert(dest);

        self.connect_ex_ex(src, dest_index);

        dest_index
    }

    /// Connects an existing node (`src`) into a new node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    ///
    /// Returns the index of `dest`.
    pub fn connect_ex_new_port<N: Node + Send + 'static>(
        &mut self,
        src: NodeIndex,
        dest: N,
        dest_port: usize,
    ) -> NodeIndex {
        let dest_index = self.insert(dest);

        self.connect_ex_ex_port(src, dest_index, dest_port);

        dest_index
    }

    /// Connects a new node (`src`) into an existing node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    ///
    /// Returns the index of `src`.
    pub fn connect_new_ex<N: Node + Send + 'static>(
        &mut self,
        src: N,
        dest: NodeIndex,
    ) -> NodeIndex {
        let src_index = self.insert(src);

        self.connect_ex_ex(src_index, dest);

        src_index
    }

    /// Connects a new node (`src`) into an existing node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    ///
    /// Returns the index of `src`.
    pub fn connect_new_ex_port<N: Node + Send + 'static>(
        &mut self,
        src: N,
        dest: NodeIndex,
        dest_port: usize,
    ) -> NodeIndex {
        let src_index = self.insert(src);

        self.connect_ex_ex_port(src_index, dest, dest_port);

        src_index
    }

    /// Connects a new node (`src`) into a new node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    ///
    /// Returns the index of (`src`, and `dest`)
    pub fn connect_new_new<N: Node + Send + 'static, O: Node + Send + 'static>(
        &mut self,
        src: N,
        dest: O,
    ) -> (NodeIndex, NodeIndex) {
        let src_index = self.insert(src);

        (src_index, self.connect_ex_new(src_index, dest))
    }

    /// Connects a new node (`src`) into a new node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    ///
    /// Returns the index of (`src`, and `dest`)
    pub fn connect_new_new_port<N: Node + Send + 'static, O: Node + Send + 'static>(
        &mut self,
        src: N,
        dest: O,
        dest_port: usize,
    ) -> (NodeIndex, NodeIndex) {
        let src_index = self.insert(src);
        let dest_index = self.insert(dest);

        self.connect_ex_ex_port(src_index, dest_index, dest_port);

        (src_index, dest_index)
    }

    /// Connects a new node, containing a constant number (`src`), into a new node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    ///
    /// Returns the index of `dest`.
    pub fn connect_const_new<N: Node + Send + 'static>(&mut self, src: f64, dest: N) -> NodeIndex {
        self.connect_new_new(c(src), dest).1
    }

    /// Connects a new node, containing a constant number (`src`), into a new node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    ///
    /// Returns the index of `dest.`
    pub fn connect_const_new_port<N: Node + Send + 'static>(
        &mut self,
        src: f64,
        dest: N,
        dest_port: usize,
    ) -> NodeIndex {
        self.connect_new_new_port(c(src), dest, dest_port).1
    }

    /// Connects a new node, containing a constant number (`src`), into an existing node (`dest`).
    /// `src` will always connect to port 0 of `dest`.
    pub fn connect_const_ex(&mut self, src: f64, dest: NodeIndex) {
        self.connect_new_ex(c(src), dest);
    }

    /// Connects a new node, containing a constant number (`src`), into an existing node (`dest`).
    /// `dest_port` determines the the port number of `dest` that `src` will connect to.
    pub fn connect_const_ex_port(&mut self, src: f64, dest: NodeIndex, dest_port: usize) {
        self.connect_new_ex_port(c(src), dest, dest_port);
    }
}

#[inline(always)]
fn update_node_inputs(
    dag: &StableDiGraph<NodeData, usize, u32>,
    node: NodeIndex,
    input_arena_ptr: usize,
    input_val_arena: &mut [Sample],
    input_arena: &mut [NodeIndex],
) -> usize {
    let node = &dag.node_weight(node).unwrap().node;
    let inputs = node.get_input_labels().len();

    for i in 0..inputs {
        input_val_arena[input_arena_ptr + i] = dag
            .node_weight(input_arena[input_arena_ptr + i])
            .unwrap()
            .val;
    }

    inputs
}
#[inline(always)]
fn would_cycle<N, E, Ix: IndexType>(
    dag: &StableDiGraph<N, E, Ix>,
    src: NodeIndex<Ix>,
    dest: NodeIndex<Ix>,
    dag_cycle_state: &mut DfsSpace<NodeIndex<Ix>, <StableDiGraph<N, E, Ix> as Visitable>::Map>,
) -> bool {
    should_check_for_cycle(dag, src, dest)
        && petgraph::algo::has_path_connecting(dag, dest, src, Some(dag_cycle_state))
}

#[inline(always)]
fn should_check_for_cycle<N, E, Ix: IndexType>(
    dag: &StableDiGraph<N, E, Ix>,
    src: NodeIndex<Ix>,
    dest: NodeIndex<Ix>,
) -> bool {
    if src == dest {
        return true;
    }

    dag.neighbors_directed(src, Incoming).next().is_some()
        && dag.neighbors_directed(dest, Outgoing).next().is_some()
        && dag.find_edge(src, dest).is_none()
}
