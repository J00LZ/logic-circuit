use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::circuit::Circuit;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ComponentError {
    #[error("On {0:?}, {1} wires were expected, but {2} were found!")]
    WrongConnectionCount(Side, usize, usize),
    #[error("A component with the id {0} was not found!")]
    MissingComponentError(Uuid),
    #[error("A wire with the id {0} was not found!")]
    MissingWireError(Uuid),
    #[error("A circuit component must have a circuit as a state!")]
    InvalidState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Side {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub(crate) component: Component,
    inputs: Vec<Uuid>,
    outputs: Vec<Uuid>,
    #[serde(skip_serializing_if = "ComponentState::is_empty")]
    #[serde(default)]
    state: ComponentState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ComponentState {
    #[default]
    Empty,
    Led(bool),
    Pins(Vec<SignalState>),
    EmbeddedCircuit(Circuit),
}

impl ComponentState {
    fn is_empty(&self) -> bool {
        matches!(self, ComponentState::Empty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum SignalState {
    High,
    Low,
    #[default]
    NotConnected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signal {
    #[serde(default)]
    #[serde(skip_serializing)]
    pub(crate) state: SignalState,
    pub(crate) subscribers: Vec<Uuid>,
}

impl Signal {
    pub(crate) fn new() -> Self {
        Self {
            state: SignalState::NotConnected,
            subscribers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Component {
    Nand,
    And,
    Or,
    Xor,
    Not,
    HighPin,
    LowPin,
    Output,
    Clock(usize, usize),
    Led,
    Pin(usize),
    EmbeddedCircuit,
}

impl Component {
    pub(crate) fn is_root(&self) -> bool {
        match self {
            Component::HighPin | Component::LowPin | Component::Clock(_, _) => true,
            _ => false,
        }
    }
}

impl ComponentInstance {
    pub fn new(component: Component) -> Self {
        Self {
            component,
            inputs: Vec::new(),
            outputs: Vec::new(),
            state: ComponentState::Empty,
        }
    }

    pub fn add_input(&mut self, wire: Uuid) {
        self.inputs.push(wire);
    }

    pub fn add_output(&mut self) -> Uuid {
        let wire = Uuid::new_v4();

        self.outputs.push(wire);

        wire
    }

    pub fn set_state(&mut self, state: ComponentState) -> ComponentState {
        let old_state = self.state.clone();
        self.state = state;
        old_state
    }

    pub fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    pub fn exec(
        &mut self,
        wires: &mut HashMap<Uuid, SignalState>,
        tick: usize,
    ) -> Result<(), ComponentError> {
        self.validate_component()?;
        match self.component {
            Component::Nand => {
                let a = wires.get(&self.inputs[0]).unwrap();
                let b = wires.get(&self.inputs[1]).unwrap();
                let out = match (a, b) {
                    (SignalState::High, SignalState::High) => SignalState::Low,
                    (SignalState::NotConnected, _) => SignalState::NotConnected,
                    (_, SignalState::NotConnected) => SignalState::NotConnected,
                    _ => SignalState::High,
                };
                wires.insert(self.outputs[0], out);
                Ok(())
            }
            Component::And => {
                let a = wires.get(&self.inputs[0]).unwrap();
                let b = wires.get(&self.inputs[1]).unwrap();
                let out = match (a, b) {
                    (SignalState::High, SignalState::High) => SignalState::High,
                    (SignalState::NotConnected, _) => SignalState::NotConnected,
                    (_, SignalState::NotConnected) => SignalState::NotConnected,
                    _ => SignalState::Low,
                };
                wires.insert(self.outputs[0], out);
                Ok(())
            }
            Component::Or => {
                let a = wires.get(&self.inputs[0]).unwrap();
                let b = wires.get(&self.inputs[1]).unwrap();
                let out = match (a, b) {
                    (SignalState::Low, SignalState::Low) => SignalState::Low,
                    (SignalState::NotConnected, _) => SignalState::NotConnected,
                    (_, SignalState::NotConnected) => SignalState::NotConnected,
                    _ => SignalState::High,
                };
                wires.insert(self.outputs[0], out);
                Ok(())
            }
            Component::Xor => {
                let a = wires.get(&self.inputs[0]).unwrap();
                let b = wires.get(&self.inputs[1]).unwrap();
                let out = match (a, b) {
                    (SignalState::High, SignalState::High) => SignalState::Low,
                    (SignalState::Low, SignalState::Low) => SignalState::Low,
                    (SignalState::NotConnected, _) => SignalState::NotConnected,
                    (_, SignalState::NotConnected) => SignalState::NotConnected,
                    _ => SignalState::High,
                };
                wires.insert(self.outputs[0], out);
                Ok(())
            }
            Component::Not => {
                let a = wires.get(&self.inputs[0]).unwrap();
                let out = match a {
                    SignalState::High => SignalState::Low,
                    SignalState::Low => SignalState::High,
                    _ => SignalState::NotConnected,
                };
                wires.insert(self.outputs[0], out);
                Ok(())
            }
            Component::HighPin => {
                wires.insert(self.outputs[0], SignalState::High);
                Ok(())
            }
            Component::LowPin => {
                wires.insert(self.outputs[0], SignalState::Low);
                Ok(())
            }
            Component::Output => {
                let a = wires.get(&self.inputs[0]).unwrap();
                println!("Output: {:?}", a);
                Ok(())
            }
            Component::Clock(off_time, on_time) => {
                let t = tick % (off_time + on_time);
                if t < on_time {
                    wires.insert(self.outputs[0], SignalState::High);
                } else {
                    wires.insert(self.outputs[0], SignalState::Low);
                }
                Ok(())
            }
            Component::Led => {
                let a = wires.get(&self.inputs[0]).unwrap();
                match a {
                    SignalState::High => self.state = ComponentState::Led(true),
                    SignalState::Low => self.state = ComponentState::Led(false),
                    SignalState::NotConnected => self.state = ComponentState::Empty,
                }
                Ok(())
            }
            Component::Pin(idx) => {
                if let ComponentState::Pins(pins) = &mut self.state {
                    let z = &mut pins[idx];
                    if self.inputs.len() == 1 {
                        *z = *wires.get(&self.inputs[0]).unwrap();
                    } else {
                        wires.insert(self.outputs[0], *z);
                    }
                }
                Ok(())
            }
            Component::EmbeddedCircuit => {
                if let ComponentState::EmbeddedCircuit(circuit) = &self.state {}
                Ok(())
            }
        }
    }

    pub(crate) fn validate_component(&self) -> Result<(), ComponentError> {
        match self.component {
            Component::Nand | Component::And | Component::Or | Component::Xor => {
                if self.inputs.len() != 2 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Input,
                        2,
                        self.inputs.len(),
                    ));
                }
                if self.outputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Output,
                        1,
                        self.outputs.len(),
                    ));
                }
                Ok(())
            }
            Component::Not => {
                if self.inputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Input,
                        1,
                        self.inputs.len(),
                    ));
                }
                if self.outputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Output,
                        1,
                        self.outputs.len(),
                    ));
                }
                Ok(())
            }
            Component::HighPin | Component::LowPin => {
                if self.outputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Output,
                        1,
                        self.outputs.len(),
                    ));
                }
                Ok(())
            }

            Component::Output => {
                if self.inputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Input,
                        1,
                        self.inputs.len(),
                    ));
                }
                Ok(())
            }
            Component::Clock(_, _) => {
                if self.outputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Output,
                        1,
                        self.outputs.len(),
                    ));
                }
                Ok(())
            }
            Component::Led => {
                if self.inputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Input,
                        1,
                        self.inputs.len(),
                    ));
                }
                Ok(())
            }
            Component::Pin(_) => {
                // a pin can have either one input or one output
                if self.inputs.len() + self.outputs.len() != 1 {
                    return Err(ComponentError::WrongConnectionCount(
                        Side::Input,
                        1,
                        self.inputs.len() + self.outputs.len(),
                    ));
                }
                Ok(())
            }
            Component::EmbeddedCircuit => {
                if let ComponentState::EmbeddedCircuit(circuit) = &self.state {
                    circuit.validate()
                } else {
                    Err(ComponentError::InvalidState)
                }
            }
        }
    }
}
