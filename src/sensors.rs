use std::collections::HashMap;

pub type SensoryPayload = HashMap<SensorTag, f32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorTag
{
    PositionX,
    PositionY,
    Orientation,
    LastMoveSucceded,
}

pub struct Sensor
{
    pub tag: SensorTag,
    pub cardinality: usize,
    index: usize,
}
impl Sensor
{
}
