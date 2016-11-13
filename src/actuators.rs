//TODO: These sould derive Clone / Copy and others
use ::types::*;
use ::serialization::*;

#[derive(Copy, Clone, Debug)]
pub enum Action
{
    NoAction,
    MoveAction(MoveAction),
}

impl ToJson for Action
{
    fn to_json(&self) -> Json 
    {
        match (*self)
        {
            Action::NoAction =>
            {
                Json::Object(pmJsonObject::new())
            },
            Action::MoveAction(MoveAction::Move(d, i)) =>
            {
                let mut json_obj = pmJsonObject::new();
                json_obj.insert("direction".to_string(), d.to_json());
                json_obj.insert("impulse".to_string(), i.to_json());
                Json::Object(json_obj)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveAction
{
    Move(Direction, f32),
}

pub type ActionList = Vec<Action>;

#[derive(Clone, Copy, Debug)]
pub enum ActuatorTag
{
    MoveHorizontal,
    MoveVertical,
}
impl ActuatorTag
{
    pub fn to_action(&self, stimulus: f32) -> Action
    {
        match *self
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

#[derive(Clone, Copy)]
pub struct Actuator
{
    tag: ActuatorTag,
    index: usize, 
}
impl Actuator
{
    pub fn new(tag: ActuatorTag, index: usize) -> Actuator
    {
        Actuator { tag: tag, index: index }
    }
    pub fn get_action(&self, stimulus: f32) -> Action
    {
        self.tag.to_action(stimulus)
    }
}
