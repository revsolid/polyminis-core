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

#[derive(Clone, Copy)]
pub struct Sensor
{
    pub tag: SensorTag,
    pub cardinality: usize,
    index: usize,
}
impl Sensor
{
    pub fn new(tag: SensorTag, index: usize) -> Sensor
    {
        //TODO: Cardinality
        Sensor { tag: tag, cardinality: 1, index: index }
    }

    pub fn get_total_cardinality(sensors: &Vec<Sensor>) -> usize
    {
        let mut in_len = 0;
        for s in sensors
        {
            in_len += s.cardinality;
        }
        in_len
    }
}
