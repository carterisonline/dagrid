# DaGrid
THIS IS NOT A FINISHED PROJECT. GO ELSEWHERE! GRAAAAAAAH

## Features :)

It can turn this code:
```rust
let mut cg = glicol::std_cg();

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
cg.connect_existing_aout(div_out[0]);
```

Into this graph:

![graph](media/subsynth_using_containers_graph.png)

And can actually output the audio sample-by-sample - that's why we're here, is it?