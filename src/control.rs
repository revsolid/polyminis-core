//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE silinapse 
extern crate silinapse;

use self::silinapse::{FeedforwardLayer, Compute};
use self::silinapse::activations::{sigmoid, ActivationFunction};
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
    //TODO: Orientation should be an enum (?)
    pub orientation: u8,
    pub last_move_succeeded: bool, 
}
impl Perspective
{
    pub fn new(id: usize, pos: (f32, f32),
               orientation: u8, move_succeded: bool) -> Perspective
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
        // TODO: Neural Network Sizing
        Control {
                  sensor_list: vec![],
                  actuator_list: vec![],
                  nn: vec![FeedforwardLayer::new(1, 1, sigmoid())],
                  inputs: vec![],
                  outputs: vec![]
                }
    }

    pub fn new_from(sensor_list: Vec<Sensor>, actuator_list: Vec<Actuator>) -> Control
    {
        let in_len = sensor_list.len();
        let out_len = actuator_list.len();
        let hid_len = 7;

        let in_to_hidden: NNLayer = FeedforwardLayer::new(in_len,hid_len,sigmoid());
        let hidden_to_out: NNLayer = FeedforwardLayer::new(hid_len, out_len, sigmoid());

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
                    println!("Sensed for tag {:?}: {}", sensor.tag, payload);  
                    self.inputs.push(*payload);
                },
                None =>
                {
                    // Error (?)
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
}
impl Genetics for Control
{
    fn crossover(&self, _: &Control, _: &mut PolyminiRandomCtx) -> Control
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

    fn mutate(&self, random_ctx: &mut PolyminiRandomCtx)
    {
        // 
        //
        let layer_to_mutate = self.nn[random_ctx.gen_range(0, self.nn.len())];

        let flip = random_ctx.test_value(0.5);
        //TODO:
        //let flip = random_ctx.test_value(1 / layer_to_mutate.get_coeffiecients().len());

        // Mutate a Weight
        if flip
        {
            let weights = layer_to_mutate.get_coefficients().clone();
            let inx = random_ctx.gen::<usize>();

        }
        // Mutate a Bias
        else
        {
        }
        
    }
}
