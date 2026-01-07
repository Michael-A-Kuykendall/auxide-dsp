//! Builder utilities for simplified DSP graph construction.

use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;

/// Builder for constructing simple synth graphs with fluent API.
///
/// Provides a convenient way to add oscillators, filters, and effects
/// to an audio graph without direct node/edge management.
pub struct SynthBuilder {
    graph: Graph,
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
        }
    }

    /// Adds an oscillator to the graph.
    pub fn add_oscillator<T: auxide::node::NodeDef + 'static>(mut self, osc: T) -> Self {
        self.graph.add_external_node(osc);
        self
    }

    /// Adds a filter to the graph.
    pub fn add_filter<T: auxide::node::NodeDef + 'static>(mut self, filter: T) -> Self {
        self.graph.add_external_node(filter);
        self
    }

    /// Adds an envelope generator to the graph.
    pub fn add_envelope<T: auxide::node::NodeDef + 'static>(mut self, env: T) -> Self {
        self.graph.add_external_node(env);
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
    pub fn build(self, block_size: usize) -> Result<(Graph, Plan), auxide::plan::PlanError> {
        let plan = Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}

/// Builder for constructing effect chains with fluent API.
pub struct EffectsChainBuilder {
    graph: Graph,
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
        }
    }

    /// Adds an input node to the chain.
    pub fn add_input(mut self) -> Self {
        self.graph.add_node(NodeType::Dummy);
        self
    }

    /// Adds an effect to the chain.
    pub fn add_effect<T: auxide::node::NodeDef + 'static>(mut self, effect: T) -> Self {
        self.graph.add_external_node(effect);
        self
    }

    /// Adds an output sink to the chain.
    pub fn add_output(mut self) -> Self {
        self.graph.add_node(NodeType::OutputSink);
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
    pub fn build(self, block_size: usize) -> Result<(Graph, Plan), auxide::plan::PlanError> {
        let plan = Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}
