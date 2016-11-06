//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE tinnmann 
extern crate tinnmann;

use self::tinnmann::{FeedforwardLayer, Compute};
use self::tinnmann::activations::{sigmoid, ActivationFunction};
//

use std::collections::HashMap;

pub use ::actuators::*;
pub use ::sensors::*;
use ::genetics::*;
use ::types::*;

pub struct Perspective
{
    pub id: usize,
    pub pos: (f32, f32),
    pub orientation: Direction,
    pub last_move_succeeded: bool, 
}
impl Perspective
{
    pub fn new(id: usize, pos: (f32, f32),
               orientation: Direction, move_succeded: bool) -> Perspective
    {
        Perspective { id: id, pos: pos, orientation: orientation,
                      last_move_succeeded: move_succeded }
    }
}

type NNLayer = FeedforwardLayer<f32, fn(f32) -> f32, fn(f32) -> f32>;
pub struct Control
{
    sensor_list: Vec<Sensor>, 
    actuator_list: Vec<Actuator>,

    // NN
    nn: Vec<NNLayer>,


    //
    inputs: Vec<f32>,
    outputs: Vec<f32>,
}
impl Control
{
    pub fn new() -> Control
    {
        Control {
                  sensor_list: vec![],
                  actuator_list: vec![],
                  nn: vec![FeedforwardLayer::new(1, 1, sigmoid())],
                  inputs: vec![],
                  outputs: vec![]
                }
    }

    pub fn new_from<T>(sensor_list: Vec<Sensor>, actuator_list: Vec<Actuator>, in_to_hid_weight_generator: &mut T, hid_to_out_weight_generator: &mut T) -> Control where T: WeightsGenerator
    {
        let mut in_len = 0;
        for s in &sensor_list
        {
            in_len += s.cardinality;
        }
        let out_len = actuator_list.len();
        let hid_len = 7;

        let in_to_hidden: NNLayer = FeedforwardLayer::new_from(in_len,hid_len,sigmoid(),     || (in_to_hid_weight_generator.generate()));
        let hidden_to_out: NNLayer = FeedforwardLayer::new_from(hid_len, out_len, sigmoid(), || (hid_to_out_weight_generator.generate()));

        Control
        {
          sensor_list: sensor_list,
          actuator_list: actuator_list,
          nn: vec![in_to_hidden, hidden_to_out],
          inputs: vec![0.0; in_len],
          outputs: vec![0.0; out_len]
        }
    }
    
    pub fn sense(&mut self, sensed: &SensoryPayload)
    {
        self.inputs.clear();
        for sensor in &self.sensor_list
        {
            match sensed.get(&sensor.tag)
            {
                Some(payload) =>
                {
                    //TODO: Multi-input sensors (e.g. Direction + Distance)
                    self.inputs.push(*payload);
                },
                None =>
                {
                    // Not an Error could be that there's nothing to be 
                    // sensed for that particular tag
                }
            }
        }
    }
    pub fn think(&mut self)
    {
        // Feedforward NN
        let mut ins : Vec<f32> = self.inputs.clone();
        for ff in &self.nn
        {
           ins = ff.compute(&ins);
        }
        self.outputs = ins.clone();
    }
    pub fn get_actions(&self) -> ActionList
    {
        // Get actions from Actuators
        // Copy actions into ActionList
        let mut action_list = vec![];
        for i in 0..self.actuator_list.len()
        {
            let ref actuator = self.actuator_list[i];
            action_list.push(actuator.get_action(self.outputs[i]));
        }
        // TODO: Return this action list
        action_list;

        // TODO: TOTALLY temporary implementation used to test
        vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2)),
             Action::MoveAction(MoveAction::Move(Direction::ROTATION, 1.1))]
    }

    pub fn crossover(&self, other: &Control, rand_ctx: &mut PolyminiRandomCtx, new_sensor_list: Vec<Sensor>, new_actuator_list: Vec<Actuator>) -> Control
    {
        Control::new()
//        {
//           sensor_list: sensor_list,
//           actuator_list: actuator_list,
//           nn: vec![input_layer, hidden_layer, output_layer],
//           inputs: vec![0.0; in_len],
//           outputs: vec![0.0; out_len]
//        }
    }

    pub fn mutate(&mut self, random_ctx: &mut PolyminiRandomCtx)
    {
        // 
        //
        let layers = self.nn.len();
        let layer_to_mutate = &mut self.nn[random_ctx.gen_range(0, layers)];

        let mut_bias = random_ctx.test_value(1 / layer_to_mutate.get_coefficients().len());

        // Mutate a Weight
        if !mut_bias
        {
            let mut weights = layer_to_mutate.get_coefficients().clone();
            let inx = random_ctx.gen_range(0, weights.len());
            weights[inx] = random_ctx.gen::<f32>();
            layer_to_mutate.set_coefficients(weights);
        }
        // Mutate a Bias
        else
        {
            let mut biases = layer_to_mutate.get_biases().clone();
            let inx = random_ctx.gen_range(0, biases.len());
            biases[inx] = random_ctx.gen::<f32>();
            layer_to_mutate.set_biases(biases);
        }
        
    }
}


pub trait WeightsGenerator
{
    fn generate(&mut self) -> f32;
}

//
struct CrossoverWeightsGenerator
{
    weights_generated: usize,
    max_weights: usize,
    weight_values: Vec<f32>,
    bias_values: Vec<f32>
}
impl CrossoverWeightsGenerator
{
    fn new(l1: NNLayer, l2: NNLayer) -> CrossoverWeightsGenerator
    {
        let mut inputs = 0;
        let mut outputs = 0;
        let mut w_values = vec![]; 
        let mut b_values = vec![]; 
        CrossoverWeightsGenerator { weights_generated: 0, max_weights: inputs*outputs + outputs, weight_values: w_values, bias_values: b_values }
    }
}

impl WeightsGenerator for CrossoverWeightsGenerator
{
    fn generate(&mut self) -> f32
    {
        if (self.weights_generated > self.max_weights)
        {
            panic!("Incorrectly set Generator");
        }

        let mut to_ret = 0.0;
        if (self.weights_generated < self.weight_values.len())
        {
            to_ret = self.weight_values[self.weights_generated];
        }
        else
        {
            let i = self.weights_generated - self.weight_values.len();
            to_ret = self.bias_values[i];
        }
        self.weights_generated += 1;
        to_ret 
    }
}

//
pub struct RandomWeightsGenerator<'a>
{
    rand_ctx: &'a mut PolyminiRandomCtx,
}
impl<'a> RandomWeightsGenerator<'a>
{
    pub fn new(ctx: &'a mut PolyminiRandomCtx) -> RandomWeightsGenerator
    {
        RandomWeightsGenerator { rand_ctx: ctx }
    }
}
impl<'a> WeightsGenerator for RandomWeightsGenerator<'a>
{
    fn generate(&mut self) -> f32
    {
        self.rand_ctx.gen::<f32>()
    }
}
