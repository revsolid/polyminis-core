//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE tinmann
// TODO: tinmann integration
//

use std::collections::HashMap;

use ::actuators::*;
use ::genetics::*;
use ::types::*;

pub struct Perspective
{
    pub id: usize,
    pub pos: (f32, f32),
    //TODO: Orientation should be an enum (?)
    pub orientation: u8,
    pub last_move_succeeded: bool, 
// list of sensor tags
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

pub type SensorTag = i32;
pub type SensoryPayload = HashMap<SensorTag, f32>;



pub struct Control
{
    sensor_list: Vec<SensorTag>, 
    actuator_list: Vec<ActuatorTag>,
    // NN
}
impl Control
{
    pub fn new() -> Control
    {
        Control { sensor_list: vec![],
                  actuator_list: vec![] }
    }
    pub fn sense(&mut self, sensed: &SensoryPayload)
    {
        for tag in &self.sensor_list
        {
            match sensed.get(&tag)
            {
                Some(payload) =>
                {
                    println!("Sensed for tag {:?}: {}", tag, payload);  
                    /* Set values into the neural network */
                },
                None =>
                {
                    // Error (?)
                }
            }
        }
    }
    pub fn think(&self)
    {
        // Feedforward NN
        // Copy values from output layer into Actuators
    }
    pub fn get_actions(&self) -> ActionList
    {
        // Get actions from Actuators
        // Copy actions into ActionList
        // TODO: TOTALLY temporary implementation used to test
        vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2)),
             Action::MoveAction(MoveAction::Move(Direction::ROTATION, 1.1))]
    }
    pub fn get_sensor_list(&self) -> &Vec<SensorTag>
    {
        &self.sensor_list
    }
}
impl Genetics for Control
{
    fn crossover(&self, _: &Control, _: &mut PolyminiRandomCtx) -> Control
    {
        Control::new()
    }

    fn mutate(&self, _: &mut PolyminiRandomCtx){}
}
