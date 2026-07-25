//! Builder utilities for simplified DSP graph construction.

use auxide::graph::{Edge, Graph, NodeId, NodeType, PortId, Rate};

/// Builder for constructing simple synth graphs with fluent API.
///
/// Nodes are chained in insertion order (oscillator -> filter -> envelope -> …)
/// and an output sink is appended automatically on `build`, so the resulting
/// graph is fully wired and ready to compile.
pub struct SynthBuilder {
    graph: Graph,
    last: Option<NodeId>,
}

impl Default for SynthBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SynthBuilder {
    /// Creates a new empty synth builder.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            last: None,
        }
    }

    fn chain(&mut self, id: NodeId) {
        if let Some(prev) = self.last {
            self.graph
                .add_edge(Edge {
                    from_node: prev,
                    from_port: PortId(0),
                    to_node: id,
                    to_port: PortId(0),
                    rate: Rate::Audio,
                })
                .expect("synth builder: failed to connect nodes");
        }
        self.last = Some(id);
    }

    /// Adds an oscillator to the graph (chained after any previously added node).
    pub fn add_oscillator<T: auxide::node::NodeDef + 'static>(mut self, osc: T) -> Self {
        let id = self.graph.add_external_node(osc);
        self.chain(id);
        self
    }

    /// Adds a filter to the graph (chained after any previously added node).
    pub fn add_filter<T: auxide::node::NodeDef + 'static>(mut self, filter: T) -> Self {
        let id = self.graph.add_external_node(filter);
        self.chain(id);
        self
    }

    /// Adds an envelope generator to the graph (chained after any previously added node).
    pub fn add_envelope<T: auxide::node::NodeDef + 'static>(mut self, env: T) -> Self {
        let id = self.graph.add_external_node(env);
        self.chain(id);
        self
    }

    /// Returns the built graph without compiling to a plan.
    pub fn build_graph(mut self) -> Graph {
        self.append_output_sink();
        self.graph
    }

    fn append_output_sink(&mut self) {
        let sink = self.graph.add_node(NodeType::OutputSink);
        if let Some(prev) = self.last {
            self.graph
                .add_edge(Edge {
                    from_node: prev,
                    from_port: PortId(0),
                    to_node: sink,
                    to_port: PortId(0),
                    rate: Rate::Audio,
                })
                .expect("synth builder: failed to connect output sink");
        }
        self.last = Some(sink);
    }

    /// Compiles the graph into an executable plan.
    ///
    /// # Arguments
    /// * `block_size` - Audio block size for the runtime
    pub fn build(
        mut self,
        block_size: usize,
    ) -> Result<(Graph, auxide::plan::Plan), auxide::plan::PlanError> {
        self.append_output_sink();
        let plan = auxide::plan::Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}

/// Builder for constructing effect chains with fluent API.
///
/// Builds `input -> effect -> effect -> … -> output` by chaining nodes in
/// insertion order and wiring edges between them.
pub struct EffectsChainBuilder {
    graph: Graph,
    last: Option<NodeId>,
}

impl Default for EffectsChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectsChainBuilder {
    /// Creates a new empty effects chain builder.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            last: None,
        }
    }

    fn chain(&mut self, id: NodeId) {
        if let Some(prev) = self.last {
            self.graph
                .add_edge(Edge {
                    from_node: prev,
                    from_port: PortId(0),
                    to_node: id,
                    to_port: PortId(0),
                    rate: Rate::Audio,
                })
                .expect("effects builder: failed to connect nodes");
        }
        self.last = Some(id);
    }

    /// Adds an input node to the chain.
    pub fn add_input(mut self) -> Self {
        let id = self.graph.add_node(NodeType::Dummy);
        self.chain(id);
        self
    }

    /// Adds an effect to the chain.
    pub fn add_effect<T: auxide::node::NodeDef + 'static>(mut self, effect: T) -> Self {
        let id = self.graph.add_external_node(effect);
        self.chain(id);
        self
    }

    /// Adds an output sink to the chain.
    pub fn add_output(mut self) -> Self {
        let id = self.graph.add_node(NodeType::OutputSink);
        self.chain(id);
        self
    }

    /// Returns the built graph without compiling to a plan.
    pub fn build_graph(self) -> Graph {
        self.graph
    }

    /// Compiles the graph into an executable plan.
    ///
    /// # Arguments
    /// * `block_size` - Audio block size for the runtime
    pub fn build(
        self,
        block_size: usize,
    ) -> Result<(Graph, auxide::plan::Plan), auxide::plan::PlanError> {
        let plan = auxide::plan::Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::filters::{SvfFilter, SvfMode};
    use crate::nodes::oscillators::SawOsc;

    #[test]
    fn effects_chain_connects_nodes() {
        let (graph, _plan) = EffectsChainBuilder::new()
            .add_input()
            .add_effect(SvfFilter {
                cutoff: 1000.0,
                resonance: 0.7,
                mode: SvfMode::Lowpass,
            })
            .add_output()
            .build(64)
            .expect("plan should compile");
        // input -> filter, filter -> output => at least 2 edges
        assert!(
            graph.edges.len() >= 2,
            "expected at least 2 edges, got {}",
            graph.edges.len()
        );
    }

    #[test]
    fn synth_builder_connects_and_outputs() {
        let (graph, _plan) = SynthBuilder::new()
            .add_oscillator(SawOsc::new(440.0))
            .build(64)
            .expect("plan should compile");
        // osc -> output sink => exactly 1 edge
        assert_eq!(graph.edges.len(), 1, "expected 1 edge (osc -> sink)");
    }
}
