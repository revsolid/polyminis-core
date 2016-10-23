use std::fmt;

use ::serialization::*;

#[derive(Copy, Clone, Debug)]
pub enum Direction
{
    UP,
    DOWN,
    LEFT,
    RIGHT,
    CLOCKWISE,
    COUNTERCLOCKWISE,

    ROTATION,
    VERTICAL,
    HORIZONTAL,
}
impl Direction
{
    pub fn to_float(&self) -> f32
    {
        match *self
        {
            Direction::UP =>    {  0.0  }
            Direction::RIGHT => {  0.25 }
            Direction::DOWN =>  {  0.5  }
            Direction::LEFT =>  {  0.75 }
            _ => { 0.0 }
        }
    }
}
impl ToJson for Direction
{
    fn to_json(&self) -> Json
    {
        Json::String(self.to_string())
    }
}
impl fmt::Display for Direction
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}
