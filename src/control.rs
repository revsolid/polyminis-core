use std::collections::HashMap;

use ::genetics::*;

pub struct ActionList {}

pub struct Perspective
{
    id: usize,
    pos: (f32, f32),
// list of sensor types
}
impl Perspective
{
    pub fn new(id: usize, pos: (f32, f32)) -> Perspective
    {
        Perspective { id: id, pos: pos }
    }
}

type SensorTag = i32;
type ActuatorTag = i32;
pub type SensoryPayload = HashMap<SensorTag, f32>;

pub struct Control
{
    // SensorList
    // ActuatorList
    // NN
}
impl Control
{
    pub fn new() -> Control
    {
        Control { }
    }
    pub fn sense(&self, _: &SensoryPayload)
    {
        // Feed SensoryPayload into sensors
        // Copy values from sensors to input layer of NN
    }
    pub fn think(&self)
    {
        // Feedforward NN
        // Copy values from output layer into Actuators
    }
    pub fn act(&self, _: &mut ActionList)
    {
        // Get actions from Actuators
        // Copy actions into ActionList
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

