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

pub type NNLayer = FeedforwardLayer<f32, fn(f32) -> f32, fn(f32) -> f32>;
pub struct Control
{
    sensor_list: Vec<Sensor>, 
    actuator_list: Vec<Actuator>,

    hidden_layer_size: usize,

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
                  hidden_layer_size: 0,
                  nn: vec![FeedforwardLayer::new(1, 1, sigmoid())],
                  inputs: vec![],
                  outputs: vec![]
                }
    }

    pub fn new_from<T>(sensor_list: Vec<Sensor>, actuator_list: Vec<Actuator>, hidden_layer_size: usize, in_to_hid_weight_generator: &mut T, hid_to_out_weight_generator: &mut T) -> Control where T: WeightsGenerator
    {
        let mut in_len = 0;
        for s in &sensor_list
        {
            in_len += s.cardinality;
        }
        let out_len = actuator_list.len();

        let in_to_hidden:  NNLayer = FeedforwardLayer::new_from(in_len, hidden_layer_size, sigmoid(), ||(in_to_hid_weight_generator.generate()));
        let hidden_to_out: NNLayer = FeedforwardLayer::new_from(hidden_layer_size, out_len, sigmoid(), ||(hid_to_out_weight_generator.generate()));

        Control
        {
          sensor_list: sensor_list,
          actuator_list: actuator_list,
          hidden_layer_size: hidden_layer_size,
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

        let hid_len = 7;
        let mut in_to_hid_generator = CrossoverWeightsGenerator::new(rand_ctx, &self.nn[0], &other.nn[0], self.inputs.len(), self.hidden_layer_size, new_sensor_list.len(), hid_len);
        let mut hid_to_out_generator = CrossoverWeightsGenerator::new(rand_ctx, &self.nn[1], &other.nn[1], self.hidden_layer_size, self.outputs.len(), hid_len, new_actuator_list.len());
        
        Control::new_from(new_sensor_list, new_actuator_list, hid_len, &mut in_to_hid_generator, &mut hid_to_out_generator)
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
        self.rand_ctx.gen_range(0.0, 1.0)
    }
}

//
//
pub struct MutateWeightsGenerator<'a>
{
    rand_ctx: &'a mut PolyminiRandomCtx,
    has_mutated: bool,
    weights_generated: usize,
    max_weights: usize,
    weight_values: Vec<f32>,
    bias_values: Vec<f32>
}
impl<'a> MutateWeightsGenerator<'a>
{
    pub fn new(ctx: &'a mut PolyminiRandomCtx) -> MutateWeightsGenerator
    {
        //TODO: Get weight_Values and bias_values from somewhere
        MutateWeightsGenerator { rand_ctx: ctx, has_mutated: false, weights_generated: 0, max_weights: 0, weight_values: vec![], bias_values: vec![] }
    }
}
impl<'a> WeightsGenerator for MutateWeightsGenerator<'a>
{
    fn generate(&mut self) -> f32
    {
        if (self.weights_generated >= self.max_weights)
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


        if !self.has_mutated
        {
            //if ( should mutate)
            {
                to_ret = self.rand_ctx.gen_range(0.0, 1.0);
            }
        }
        to_ret 
    }
}

//
//
pub struct CrossoverWeightsGenerator
{
    weights_generated: usize,
    max_weights:       usize,
    weight_values:     Vec<f32>,
    bias_values:       Vec<f32>,
}
impl CrossoverWeightsGenerator
{
    fn new<'a>(rand_ctx: &'a mut PolyminiRandomCtx, l1: &NNLayer, l2: &NNLayer, old_in_size: usize, old_out_size: usize,
               new_in_size: usize, new_out_size: usize) -> CrossoverWeightsGenerator
    {
        let mut w_values = vec![];
        let mut b_values = vec![]; 
        let mut parent1_inx: usize = 0;
        let mut parent2_inx: usize = 0;

        for i in 0..new_in_size
        {
            for j in 0..new_out_size
            {
                let v;
                if i < old_in_size && j < old_out_size
                {
                    v = l1.get_coefficients()[parent1_inx];
                    parent1_inx += 1;
                }
                else if parent2_inx < l2.get_coefficients().len()
                {
                    v = l2.get_coefficients()[parent2_inx];
                    parent2_inx += 1;
                }
                else
                {
                    v = rand_ctx.gen_range(0.0, 1.0);
                }
                w_values.push(v);
            }
        }

        for o in 0..new_out_size
        {
            if o < l1.get_biases().len()
            {
                b_values.push(l1.get_biases()[o]);
            }
            else if o - l1.get_biases().len() < l2.get_biases().len()
            {
                b_values.push(l2.get_biases()[o - l1.get_biases().len()]);
            }
            else
            {
                b_values.push(rand_ctx.gen_range(0.0, 1.0));
            }
        }


        // max_weights =  Inputs * Outpus + Biases
        CrossoverWeightsGenerator { weights_generated: 0, max_weights: new_in_size*new_out_size + new_out_size, weight_values: w_values, bias_values: b_values }

    }
}

impl WeightsGenerator for CrossoverWeightsGenerator
{
    fn generate(&mut self) -> f32
    {
        if (self.weights_generated >= self.max_weights)
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



#[cfg(test)]
mod test
{
    extern crate tinnmann;
    use self::tinnmann::{FeedforwardLayer, Compute};
    use self::tinnmann::activations::{sigmoid, ActivationFunction};

    use super::*;

    use ::genetics::*;

    struct FromValuesWeightsGenerator
    {
        w_values: Vec<f32>,
        b_values: Vec<f32>,
        weights_generated: usize,
    }
    impl WeightsGenerator for FromValuesWeightsGenerator
    {
        fn generate(&mut self) -> f32
        {
            let to_ret;

            if (self.weights_generated < self.w_values.len())
            {
                to_ret = self.w_values[self.weights_generated];
            }
            else
            {
                to_ret = self.b_values[self.weights_generated - self.w_values.len()];
            }
            self.weights_generated += 1;

            to_ret
        }
    }

    fn test_control_crossover_master(old_in_size: usize, old_out_size: usize, new_in_size: usize, new_out_size: usize, other_in_size: usize, other_out_size: usize)
    {
        let mut w_1:Vec<f32> = vec![];
        let mut b_1:Vec<f32> = vec![];
        let mut w_2:Vec<f32> = vec![];
        let mut b_2:Vec<f32> = vec![];


        for i in 0..old_in_size
        {
            for j in 0..old_out_size
            {
                w_1.push( 1.0 + i as f32 + ( (1.0 + j as f32) / 10.0));
            }
        }

        for j in 0..old_out_size
        {
            b_1.push(8.0 + ( (1.0 + j as f32) / 10.0));
        }

        for i in 0..other_in_size
        {
            for j in 0..other_out_size
            {
                w_2.push( 1.0 + i as f32 + ( (1.0 + j as f32) / 10.0) + 0.01 );
            }
        }

        for j in 0..old_out_size
        {
            b_2.push(8.0 + ( (1.0 + j as f32) / 10.0) + 0.01);
        }

        let nn_1: NNLayer = FeedforwardLayer::new_from_values(old_in_size, old_out_size, sigmoid(), w_1, b_1);
        let nn_2: NNLayer = FeedforwardLayer::new_from_values(other_in_size, other_out_size, sigmoid(), w_2, b_2);


        let mut ctx = PolyminiRandomCtx::new_unseeded("Control Tests".to_string());

        let mut crossover_generator = CrossoverWeightsGenerator::new(&mut ctx, &nn_1, &nn_2, old_in_size, old_out_size, new_in_size, new_out_size);

        let coeffs:Vec<f32> = (0..new_in_size*new_out_size).map(|_| crossover_generator.generate()).collect();
        let biases:Vec<f32> = (0..new_out_size).map(|_| crossover_generator.generate()).collect();


        for v in coeffs
        {
            println!("{}", v);
        }

        for b in biases 
        {
            println!("{}", b);
        }
    }


    #[test]
    fn test_controlcrossover_1()
    {
        test_control_crossover_master(2, 2, 2, 2, 2, 2);
    }

    #[test]
    fn test_controlcrossover_2()
    {
        test_control_crossover_master(2, 2, 3, 3, 2, 2);
    }


    #[test]
    fn test_controlcrossover_3()
    {
        test_control_crossover_master(3, 2, 2, 5, 1, 4);
    }
}
