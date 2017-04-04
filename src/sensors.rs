use std::collections::HashMap;
use std::fmt;
use ::serialization::*;

pub type SensoryPayload = HashMap<SensorTag, f32>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorTag
{
    // Default Sensors Not Evolvable
    // -- Movement
    PositionX,
    PositionY,
    Orientation,
    LastMoveSucceded,
    // --Time
    TimeGlobal,
    TimeSubStep,

    // Evolvable Sensors
    // -- FoodSources
    GSensor,

}
impl SensorTag
{
    pub fn get_cardinality(&self) -> usize
    {
        match self
        {
            _ => { 1 },
        }
    }
}
impl Serializable for SensorTag
{
    fn serialize(&self, _:&mut SerializationCtx) -> Json
    {
        self.to_string().to_json()
    }
}
impl Deserializable for SensorTag
{
    fn new_from_json(json: &Json, _: &mut SerializationCtx) -> Option<SensorTag>
    {
        let to_ret;
        match *json 
        {
            Json::String(ref json_string) =>
            {
                match json_string.to_lowercase().as_str()
                {

                    "positionx"        => { to_ret = SensorTag::PositionX },
                    "positiony"        => { to_ret = SensorTag::PositionY },
                    "orientation"      => { to_ret = SensorTag::Orientation },
                    "lastmovesucceded" => { to_ret = SensorTag::LastMoveSucceded },
                    "gsensor"          => { to_ret = SensorTag::GSensor },
                    "timeglobal"       => { to_ret = SensorTag::TimeGlobal },
                    "timesubstep"      => { to_ret = SensorTag::TimeSubStep },

                    //Default
                    _                  => { return None },
                }
            },
            _ =>
            {
                return None
            }
        }
        Some(to_ret)
    }
}
impl fmt::Display for SensorTag 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
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
        Sensor { tag: tag, cardinality: tag.get_cardinality(), index: index }
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
