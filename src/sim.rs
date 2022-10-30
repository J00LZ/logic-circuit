use std::collections::BinaryHeap;

use uuid::Uuid;

use crate::{component::{ComponentError, ComponentState}, circuit::{ComponentReference, Circuit}};

pub struct Sim {
    circuit: Circuit,
    queue: BinaryHeap<SimEvent>,
}

impl Sim {
    pub fn new(circuit: Circuit) -> Self {
        let mut queue = BinaryHeap::new();

        queue.extend(circuit.components.iter().filter_map(|(a, b)| {
            if b.component.is_root() {
                Some(SimEvent {
                    time: 0,
                    component: *a,
                })
            } else {
                None
            }
        }));
        Self { circuit, queue }
    }

    pub fn get_state(&self, r: ComponentReference) -> Result<ComponentState, ComponentError> {
        self.circuit.get_state(r)
    }

    pub fn step(&mut self, tick: usize) -> Result<(), ComponentError> {
        while let Some(event) = self.queue.pop() {
            let component = self
                .circuit
                .components
                .get_mut(&event.component)
                .ok_or_else(|| ComponentError::MissingComponentError(event.component))?;
            let mut wiremap = self
                .circuit
                .wires
                .iter()
                .map(|(a, b)| (*a, b.state))
                .collect();
            component.exec(&mut wiremap, tick)?;
            for (k, new_v) in wiremap {
                let wire = self
                    .circuit
                    .wires
                    .get_mut(&k)
                    .ok_or_else(|| ComponentError::MissingWireError(k))?;
                if wire.state != new_v {
                    wire.state = new_v;
                    for sub in &wire.subscribers {
                        self.queue.push(SimEvent {
                            time: event.time + 1,
                            component: *sub,
                        });
                    }
                }
            }
        }
        self.queue
            .extend(self.circuit.components.iter().filter_map(|(a, b)| {
                if b.component.is_root() {
                    Some(SimEvent {
                        time: 0,
                        component: *a,
                    })
                } else {
                    None
                }
            }));
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SimEvent {
    time: usize,
    component: Uuid,
}

impl PartialOrd for SimEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}
