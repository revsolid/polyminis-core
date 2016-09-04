use rust_monster::ga::ga_random::*;

use ::genetics::Genetics;

pub struct SensoryPayload{}
pub struct ActionList {}
pub struct Sensor
{
}
pub struct Actuator
{
}
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
    fn crossover(&self, _: &Control, _: &mut GARandomCtx) -> Control
    {
        Control::new()
    }

    fn mutate(&self, _: &mut GARandomCtx){}
}

