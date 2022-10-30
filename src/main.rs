mod circuit;
mod component;
mod sim;

use component::*;


use crate::{circuit::Circuit, sim::Sim};

fn main() {
    let mut c = Circuit::new();
    let nand = c.add_component(Component::Nand);
    let input1 = c.add_component(Component::HighPin);
    let input2 = c.add_component(Component::Clock(1, 1));
    let output = c.add_component(Component::Output);
    let output2 = c.add_component(Component::Led);

    let i = c.add_output(input1).unwrap();
    let i2 = c.add_output(input2).unwrap();
    c.add_input(nand, i).unwrap();
    c.add_input(nand, i2).unwrap();
    let o = c.add_output(nand).unwrap();
    c.add_input(output, o).unwrap();
    c.add_input(output2, o).unwrap();
    let json_data = serde_json::to_string_pretty(&c).unwrap();
    println!("{}", json_data);
    let c: Circuit = serde_json::from_str(&json_data).unwrap();

    let mut sim = Sim::new(c);
    for i in 0..10 {
        sim.step(i).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::{Component, ComponentState, Sim};

    macro_rules! component_test {
        ($name:ident, $lhs:expr, $rhs:expr, $res:expr, $gate:expr) => {
            #[test]
            fn $name() {
                use super::circuit::Circuit;
                let mut c = Circuit::new();
                let gate = c.add_component($gate);
                let lhs = c.add_component($lhs);
                let rhs = c.add_component($rhs);
                let output = c.add_component(Component::Led);
                let lhs_o = c.add_output(lhs).unwrap();
                let rhs_o = c.add_output(rhs).unwrap();
                c.add_input(gate, lhs_o).unwrap();
                c.add_input(gate, rhs_o).unwrap();
                let out = c.add_output(gate).unwrap();
                c.add_input(output, out).unwrap();
                let mut sim = Sim::new(c);
                sim.step(0).unwrap();
                assert_eq!(sim.get_state(output).unwrap(), $res);
            }
        };
    }

    component_test!(
        nand_test,
        Component::HighPin,
        Component::HighPin,
        ComponentState::Led(false),
        Component::Nand
    );
    component_test!(
        and_test,
        Component::HighPin,
        Component::HighPin,
        ComponentState::Led(true),
        Component::And
    );
    component_test!(
        or_test,
        Component::HighPin,
        Component::LowPin,
        ComponentState::Led(true),
        Component::Or
    );
    component_test!(
        xor_test,
        Component::HighPin,
        Component::HighPin,
        ComponentState::Led(false),
        Component::Xor
    );
}
