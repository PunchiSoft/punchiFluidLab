// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Circuit graph: components joined by connections.

use crate::{Component, CoreError, PortId};

/// Identifier of a node in the circuit graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub String);

/// One edge of the circuit: two ports that carry the same fluid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// Component and port on one side.
    pub from: (String, PortId),
    /// Component and port on the other side.
    pub to: (String, PortId),
}

/// A complete circuit, ready to be handed to the solver.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Circuit {
    /// Placed components, in insertion order.
    pub components: Vec<Component>,
    /// Connections between their ports.
    pub connections: Vec<Connection>,
}

impl Circuit {
    /// Adds a component, replacing any previous one with the same id.
    pub fn add_component(&mut self, component: Component) {
        self.components
            .retain(|existing| existing.id != component.id);
        self.components.push(component);
    }

    /// Connects two ports.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::SelfConnection`] when both ports belong to the same
    /// component, and [`CoreError::UnknownPort`] when a port does not exist.
    pub fn connect(
        &mut self,
        from: (String, PortId),
        to: (String, PortId),
    ) -> Result<(), CoreError> {
        if from.0 == to.0 {
            return Err(CoreError::SelfConnection);
        }
        for (component_id, port_id) in [&from, &to] {
            let component = self
                .components
                .iter()
                .find(|candidate| candidate.id == *component_id)
                .ok_or_else(|| CoreError::UnknownPort {
                    component: component_id.clone(),
                    port: port_id.as_str().to_owned(),
                })?;
            if !component
                .kind
                .ports()
                .iter()
                .any(|port| port.id == *port_id)
            {
                return Err(CoreError::UnknownPort {
                    component: component_id.clone(),
                    port: port_id.as_str().to_owned(),
                });
            }
        }
        self.connections.push(Connection { from, to });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ComponentKind;
    use crate::units::{Pressure, Volume};

    fn supply(id: &str) -> Component {
        Component {
            id: id.to_owned(),
            name: id.to_owned(),
            kind: ComponentKind::Supply {
                pressure: Pressure::from_bar(6.0).expect("6 bar is valid"),
            },
        }
    }

    fn reservoir(id: &str) -> Component {
        Component {
            id: id.to_owned(),
            name: id.to_owned(),
            kind: ComponentKind::Reservoir {
                volume: Volume::from_litre(1.0).expect("1 l is valid"),
            },
        }
    }

    #[test]
    fn connects_two_components() {
        let mut circuit = Circuit::default();
        circuit.add_component(supply("p"));
        circuit.add_component(reservoir("r"));

        circuit
            .connect(
                (String::from("p"), PortId::new("1")),
                (String::from("r"), PortId::new("1")),
            )
            .expect("ports exist");

        assert_eq!(circuit.connections.len(), 1);
    }

    #[test]
    fn rejects_self_connection() {
        let mut circuit = Circuit::default();
        circuit.add_component(reservoir("r"));

        let result = circuit.connect(
            (String::from("r"), PortId::new("1")),
            (String::from("r"), PortId::new("1")),
        );

        assert!(matches!(result, Err(CoreError::SelfConnection)));
    }

    #[test]
    fn rejects_unknown_port() {
        let mut circuit = Circuit::default();
        circuit.add_component(supply("p"));
        circuit.add_component(reservoir("r"));

        let result = circuit.connect(
            (String::from("p"), PortId::new("9")),
            (String::from("r"), PortId::new("1")),
        );

        assert!(matches!(result, Err(CoreError::UnknownPort { .. })));
    }
}
