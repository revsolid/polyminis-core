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
