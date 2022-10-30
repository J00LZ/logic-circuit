mod component;
use std::collections::HashMap;

use component::*;
use uuid::Uuid;

fn main() {
    let mut nand = ComponentInstance::new(Component::Nand);
    let mut wires = HashMap::new();
    let wire1 = Uuid::new_v4();
    let wire2 = Uuid::new_v4();
    let output = nand.add_output();
    nand.add_input(wire1);
    nand.add_input(wire2);
    wires.insert(wire1, SignalState::High);
    wires.insert(wire2, SignalState::High);
    nand.exec(&mut wires, 0).unwrap();
    println!("{:?}", wires.get(&output));

}
