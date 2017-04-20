//TODO: These sould derive Clone / Copy and others
use std::fmt;
use ::types::*;
use ::serialization::*;

#[derive(Copy, Clone, Debug)]
pub enum Action
{
    NoAction,
    MoveAction(MoveAction),
    ThermalAction(ThermalAction),
    PhAction(PhAction),
}

impl ToJson for Action
{
    fn to_json(&self) -> Json 
    {
        let mut json_obj = pmJsonObject::new();
        match *self
        {
            Action::NoAction =>
            {
                //
            },
            Action::MoveAction(MoveAction::Move(d, i, _)) =>
            {
                json_obj.insert("direction".to_owned(), d.to_json());
                json_obj.insert("impulse".to_owned(), i.to_json());
            },
            Action::ThermalAction(ThermalAction::Change(d)) =>
            {
                json_obj.insert("delta".to_owned(), d.to_json());
            },
            Action::PhAction(PhAction::Change(d)) =>
            {
                json_obj.insert("delta".to_owned(), d.to_json());
            },
        }
        Json::Object(json_obj)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveAction
{
    Move(Direction, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub enum ThermalAction
{
    Change(f32), 
}


#[derive(Clone, Copy, Debug)]
pub enum PhAction
{
    Change(f32), 
}

pub type ActionList = Vec<Action>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActuatorTag
{
    MoveHorizontal,
    MoveVertical,
    Thermo,
    Ph,
}
impl ActuatorTag
{
    pub fn to_action(&self, stimulus_p: f32, coord: Coord) -> Action
    {
        let mut stimulus = stimulus_p;
        if stimulus > 1.0
        {
            stimulus = 1.0;
        }
        if stimulus < -1.0
        {
            stimulus = -1.0;
        }
        match *self
        {
            ActuatorTag::MoveHorizontal =>
            {
                let torque = coord.1 as f32 * stimulus;
                Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, stimulus, torque))
            },
            ActuatorTag::MoveVertical =>
            {
                let torque = coord.0 as f32 * stimulus;
                Action::MoveAction(MoveAction::Move(Direction::VERTICAL, stimulus, torque))
            },
            ActuatorTag::Thermo =>
            {
                Action::ThermalAction(ThermalAction::Change(stimulus))
            }
            ActuatorTag::Ph =>
            {
                Action::PhAction(PhAction::Change(stimulus))
            }
        }
    }
}
impl Serializable for ActuatorTag
{
    fn serialize(&self, _:&mut SerializationCtx) -> Json
    {
        self.to_string().to_json()
    }
}
impl Deserializable for ActuatorTag
{
    fn new_from_json(json: &Json, _:&mut SerializationCtx) -> Option<ActuatorTag>
    {
        let to_ret;
        match *json 
        {
            Json::String(ref json_string) =>
            {
                match json_string.to_lowercase().as_ref()
                {
                    "hormov" => { to_ret = ActuatorTag::MoveHorizontal; }, 
                    "vermov" => { to_ret = ActuatorTag::MoveVertical; }, 
                    "thermo" => { to_ret = ActuatorTag::Thermo; }, 
                    "ph"     => { to_ret = ActuatorTag::Ph; }, 
                    _ =>
                    {
                        return None;
                    }
                }
            },
            _ =>
            {
                error!("Incorrect type passed - {:?}", json);
                return None;
            }
        }
        Some(to_ret)
    }
}
impl fmt::Display for ActuatorTag 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy)]
pub struct Actuator
{
    tag: ActuatorTag,
    index: usize, 
    coords: Coord,
}
impl Actuator
{
    pub fn new(tag: ActuatorTag, index: usize, coords: Coord) -> Actuator
    {
        Actuator { tag: tag, index: index, coords: coords}
    }
    pub fn get_action(&self, stimulus: f32) -> Action
    {
        self.tag.to_action(stimulus, self.coords)
    }
}
