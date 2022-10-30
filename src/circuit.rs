use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::component::{Component, ComponentError, ComponentInstance, ComponentState, Signal};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Circuit {
    pub(crate) components: HashMap<Uuid, ComponentInstance>,
    pub(crate) wires: HashMap<Uuid, Signal>,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            wires: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, component: Component) -> ComponentReference {
        let instance = ComponentInstance::new(component);
        let id = Uuid::new_v4();
        self.components.insert(id, instance);
        ComponentReference(id)
    }

    pub fn add_output(&mut self, r: ComponentReference) -> Result<WireReference, ComponentError> {
        let out = self
            .components
            .get_mut(&r.0)
            .ok_or_else(|| ComponentError::MissingComponentError(r.0))?
            .add_output();
        self.wires.insert(out, Signal::new());
        Ok(WireReference(out))
    }

    pub fn add_input(
        &mut self,
        r: ComponentReference,
        wire: WireReference,
    ) -> Result<(), ComponentError> {
        self.components
            .get_mut(&r.0)
            .ok_or_else(|| ComponentError::MissingComponentError(r.0))?
            .add_input(wire.0);
        self.wires
            .get_mut(&wire.0)
            .ok_or_else(|| ComponentError::MissingWireError(wire.0))?
            .subscribers
            .push(r.0);
        Ok(())
    }

    pub fn set_state(
        &mut self,
        r: ComponentReference,
        state: ComponentState,
    ) -> Result<ComponentState, ComponentError> {
        Ok(self
            .components
            .get_mut(&r.0)
            .ok_or_else(|| ComponentError::MissingComponentError(r.0))?
            .set_state(state))
    }

    pub fn get_state(&self, r: ComponentReference) -> Result<ComponentState, ComponentError> {
        Ok(self
            .components
            .get(&r.0)
            .ok_or_else(|| ComponentError::MissingComponentError(r.0))?
            .get_state())
    }

    pub(crate) fn validate(&self) -> Result<(), ComponentError> {
        self.components
            .values()
            .try_for_each(|c| c.validate_component())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentReference(Uuid);

#[derive(Debug, Clone, Copy)]
pub struct WireReference(Uuid);
