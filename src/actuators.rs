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

#[derive(Debug, Clone, Copy)]
pub enum ActuatorTag
{
    MoveHorizontal,
    MoveVertical,
}


pub struct Actuator
{
    tag: ActuatorTag,
    index: usize, 
}
impl Actuator
{
    pub fn get_action(&self, stimulus: f32) -> Action
    {
        match self.tag
        {
            ActuatorTag::MoveHorizontal =>
            {
                Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, stimulus))
            },
            ActuatorTag::MoveVertical =>
            {
                Action::MoveAction(MoveAction::Move(Direction::VERTICAL, stimulus))
            },
        }
    }
}
