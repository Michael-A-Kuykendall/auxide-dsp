use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;

/// SynthBuilder for building simple synth graphs
pub struct SynthBuilder {
    graph: Graph,
}

impl SynthBuilder {
    pub fn new() -> Self {
        Self { graph: Graph::new() }
    }

    pub fn add_oscillator<T: auxide::node::NodeDef + 'static>(mut self, osc: T) -> Self {
        self.graph.add_external_node(osc);
        self
    }

    pub fn add_filter<T: auxide::node::NodeDef + 'static>(mut self, filter: T) -> Self {
        self.graph.add_external_node(filter);
        self
    }

    pub fn add_envelope<T: auxide::node::NodeDef + 'static>(mut self, env: T) -> Self {
        self.graph.add_external_node(env);
        self
    }

    pub fn build_graph(self) -> Graph {
        self.graph
    }

    pub fn build(self, block_size: usize) -> Result<(Graph, Plan), auxide::plan::PlanError> {
        let plan = Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}

/// EffectsChainBuilder for building effect chains
pub struct EffectsChainBuilder {
    graph: Graph,
}

impl EffectsChainBuilder {
    pub fn new() -> Self {
        Self { graph: Graph::new() }
    }

    pub fn add_input(mut self) -> Self {
        self.graph.add_node(NodeType::Dummy);
        self
    }

    pub fn add_effect<T: auxide::node::NodeDef + 'static>(mut self, effect: T) -> Self {
        self.graph.add_external_node(effect);
        self
    }

    pub fn add_output(mut self) -> Self {
        self.graph.add_node(NodeType::OutputSink);
        self
    }

    pub fn build_graph(self) -> Graph {
        self.graph
    }

    pub fn build(self, block_size: usize) -> Result<(Graph, Plan), auxide::plan::PlanError> {
        let plan = Plan::compile(&self.graph, block_size)?;
        Ok((self.graph, plan))
    }
}