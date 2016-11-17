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
impl Serializable for Instinct
{
    fn serialize(&self, _:&mut SerializationCtx) -> Json
    {
        self.to_string().to_json()
    }
}
impl fmt::Display for Instinct 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}
