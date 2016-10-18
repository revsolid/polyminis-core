//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE serialize 
extern crate rustc_serialize;
pub use self::rustc_serialize::json::{self, Json, ToJson};
//
//

pub type pmJsonObject = json::Object;
pub type pmJsonArray = json::Array;

#[derive(Clone, Copy)]
pub enum SerializationMode
{
    Basic,
    SimulationStep
}
pub struct SerializationCtx
{
    mode: SerializationMode,
}
impl SerializationCtx
{
    pub fn new() -> SerializationCtx
    {
        SerializationCtx { mode: SerializationMode::Basic }
    }

    pub fn get_mode(&self) -> SerializationMode
    {
        self.mode
    }
}

pub trait Serializable
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json;
}

impl ToJson for Serializable
{
    fn to_json(&self)-> Json
    {
        self.serialize(&mut SerializationCtx::new())
    }
}
