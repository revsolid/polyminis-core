use std::collections::HashMap;
use std::fmt;
use ::serialization::*;

pub type SensoryPayload = HashMap<SensorTag, f32>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SensorTag
{
    // Default Sensors Not Evolvable
    PositionX,
    PositionY,
    Orientation,
    LastMoveSucceded,


    // Evolvable Sensors
    GSensor,

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
                match json_string.as_ref()
                {

                    "positionx"        => { to_ret = SensorTag::PositionX },
                    "positiony"        => { to_ret = SensorTag::PositionY },
                    "orientation"      => { to_ret = SensorTag::Orientation },
                    "lastmovesucceded" => { to_ret = SensorTag::LastMoveSucceded },
                    "gsensor"          => { to_ret = SensorTag::GSensor },

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
