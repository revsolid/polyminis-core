use std::collections::HashMap;

pub type SensorTag = i32;
pub type SensoryPayload = HashMap<SensorTag, f32>;

pub struct Sensor
{
    pub tag: SensorTag,
    index: usize,
}
impl Sensor
{
}
