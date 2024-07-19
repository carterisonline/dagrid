use crate::container::*;
use crate::control::ControlGraph;
use crate::node::*;

pub fn preset<F: Fn(&mut ControlGraph)>(sample_rate: u32, f: F) -> ControlGraph {
    let mut cg = ControlGraph::new(sample_rate);
    f(&mut cg);

    cg
}

pub fn subsynth_plain(cg: &mut ControlGraph) {
    // Create 440hz and 220hz oscillators
    let sine_osc_1 = cg.connect_const_new(440.0, Sine);
    let sine_osc_2 = cg.connect_const_new(220.0, Sine);

    // Subtract oscillator 1 from oscillator 2
    let mulneg1 = cg.connect_const_new(-1.0, Mul);
    cg.connect(sine_osc_2, mulneg1, 1);
    let add = cg.connect_many_new(&[mulneg1, sine_osc_1], Add);

    // Audio out must be in range (-1 < x < 1)
    // divide by 2 to avoid exceeding that
    let mulhalf = cg.connect_const_new(0.5, Mul);
    cg.connect(add, mulhalf, 1);

    // Connect product to audio output
    cg.connect_ex_aout(mulhalf);
}

pub fn subsynth_with_containers(cg: &mut ControlGraph) {
    // Create 440hz and 220hz oscillators
    let sine_osc_1 = cg.connect_const_new(440.0, Sine);
    let sine_osc_2 = cg.connect_const_new(220.0, Sine);

    // Subtract oscillator 1 from oscillator 2
    let (sub_in, sub_out) = cg.insert_container(Sub);
    cg.connect_ex_ex(sine_osc_1, sub_in[0]);
    cg.connect_ex_ex(sine_osc_2, sub_in[1]);

    // Audio out must be in range (-1 < x < 1)
    // divide by 2 to avoid exceeding that
    let (div_in, div_out) = cg.insert_container(Div);
    cg.connect_ex_ex(sub_out[0], div_in[0]);
    cg.connect_const_ex(2.0, div_in[1]);

    // Connect product to audio output
    cg.connect_ex_aout(div_out[0]);
}

pub fn subsynth_plain_multiout(cg: &mut ControlGraph) {
    // Create a 220.0 const to plug in to both oscillators
    let c_220 = cg.insert(c(220.0));

    let sine_osc_1 = cg.connect_ex_new(c_220, Sine);
    let sine_osc_2 = cg.connect_ex_new(c_220, Sine);
    let sine_osc_3 = cg.connect_ex_new(sine_osc_2, Sine);

    // Add oscillator 1 to oscillator 2
    let add = cg.connect_many_new(&[sine_osc_1, sine_osc_3], Add);

    // Audio out must be in range (-1 < x < 1)
    // divide by 2 to avoid exceeding that
    let mulhalf = cg.connect_const_new(0.5, Mul);
    cg.connect(add, mulhalf, 1);

    // Connect product to audio output
    cg.connect_ex_aout(mulhalf);
}
