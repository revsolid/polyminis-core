use std::fmt;

use ::serialization::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instinct
{
    Basic,
    Herding,
    Hoarding,
    Nomadic,
    Predatory,
}
impl Instinct
{
    pub fn from_string(string: &String) -> Option<Instinct>
    {
        match string.to_lowercase().as_str()
        {
            "basic" =>
            {
                Some(Instinct::Basic)
            },
            "herding" =>
            {
                Some(Instinct::Herding)
            },
            "hoarding" =>
            {
                Some(Instinct::Hoarding)
            },
            "nomadic" =>
            {
                Some(Instinct::Nomadic)
            },
            "predatory" =>
            {
                Some(Instinct::Predatory)
            },
            _ =>
            {
                None
            },
        }
    }
}
impl Serializable for Instinct
{
    fn serialize(&self, _:&mut SerializationCtx) -> Json
    {
        self.to_string().to_lowercase().to_json()
    }
}
impl Deserializable for Instinct
{
    fn new_from_json(json: &Json, _: &mut SerializationCtx) -> Option<Instinct> 
    {
        match json 
        {
            &Json::String(ref json_string) =>
            {
                Instinct::from_string(json_string)
            },
            _ =>
            {
                None
            }
        }
    } 
}
impl fmt::Display for Instinct 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}
