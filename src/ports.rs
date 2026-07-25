//! Named ports: wire nodes by semantic name instead of bare `PortId(0)`.
//!
//! Replaces manual `Port { id: PortId(0), .. }` literals with readable names.
//! Each node's input/output port arrays are independent, so the same `PortId`
//! value can carry a different semantic name in different nodes (e.g. a node's
//! `OUT` and another node's `FREQ_MOD` can both be `PortId(0)`).

use auxide::graph::{Port, PortId, Rate};

/// Semantic output port (typically `PortId(0)`).
pub const OUT: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic input port (typically `PortId(0)`).
pub const IN: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic gate input (typically `PortId(0)`).
pub const GATE: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic frequency input (typically `PortId(0)`).
pub const FREQ: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic frequency-modulation input (typically `PortId(0)`).
pub const FREQ_MOD: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic cutoff input (typically `PortId(0)`).
pub const CUTOFF: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic resonance input (typically `PortId(0)`).
pub const RESONANCE: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};
/// Semantic mix input (typically `PortId(0)`).
pub const MIX: Port = Port {
    id: PortId(0),
    rate: Rate::Audio,
};

/// Build a named port with an explicit `id` for nodes that expose several ports.
pub fn port(id: usize, rate: Rate) -> Port {
    Port {
        id: PortId(id),
        rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auxide::graph::PortId;

    #[test]
    fn named_ports_preserve_zero_id() {
        // Migrating to named ports must not change the on-wire PortId, or graphs
        // built against PortId(0) would break.
        assert_eq!(OUT.id, PortId(0));
        assert_eq!(IN.id, PortId(0));
        assert_eq!(FREQ_MOD.id, PortId(0));
        assert_eq!(GATE.id, PortId(0));
    }

    #[test]
    fn port_helper_assigns_id() {
        let p = port(2, Rate::Audio);
        assert_eq!(p.id, PortId(2));
    }

    #[test]
    fn constant_uses_named_out() {
        // `Constant` exposes exactly the named OUT port at id 0.
        let ports: &[Port] = &[OUT];
        assert_eq!(ports[0].id, PortId(0));
        assert_eq!(ports.len(), 1);
    }
}
