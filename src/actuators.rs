//TODO: These sould derive Clone / Copy and others
use ::types::*;

#[derive(Debug)]
pub enum Action
{
    NoAction,
    MoveAction(MoveAction),
}

#[derive(Debug)]
pub enum MoveAction
{
    Move(Direction, f32),
}

pub type ActionList = Vec<Action>;
pub type ActuatorTag = i32;

pub struct Actuator
{
    tag: ActuatorTag,
    index: usize, 
}
