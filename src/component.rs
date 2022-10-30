use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Component {
    Nand,
    HighPin,
    LowPin,
    Output,
}

pub struct ComponentInstance {
    component: Component,
    inputs: Vec<Uuid>,
    outputs: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalState {
    High,
    Low,
    Unknown,
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ComponentError {
    #[error("A wire with the id {0} was not found!")]
    MissingWireError(Uuid),
}

impl ComponentInstance {
    pub fn new(component: Component) -> Self {
        Self {
            component,
            inputs: Vec::new(),
            outputs: Vec::new(),
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

    pub fn exec(
        &self,
        wires: &mut HashMap<Uuid, SignalState>,
        tick: usize,
    ) -> Result<(), ComponentError> {
        match self.component {
            Component::Nand => {
                let a = wires
                    .get(&self.inputs[0])
                    .ok_or(ComponentError::MissingWireError(self.inputs[0]))?;
                let b = wires
                    .get(&self.inputs[1])
                    .ok_or(ComponentError::MissingWireError(self.inputs[1]))?;

                let result = match (a, b) {
                    (SignalState::High, SignalState::High) => SignalState::Low,
                    _ => SignalState::High,
                };

                wires.insert(self.outputs[0], result);
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
                let a = wires
                    .get(&self.inputs[0])
                    .ok_or(ComponentError::MissingWireError(self.inputs[0]))?;

                println!("Tick {}: {:?}", tick, a);
                Ok(())
            }
        }
    }
}
