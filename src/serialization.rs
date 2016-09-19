//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE serialize 
extern crate rustc_serialize;
pub use self::rustc_serialize::json::{self, Json, ToJson};
//
//

pub type pmJsonObject = json::Object;
pub type pmJsonArray = json::Array;

enum SerializationState
{
    Top,
    Object,
    List,
}
pub struct SerializationCtx
{
}
impl SerializationCtx
{
    pub fn new() -> SerializationCtx
    {
        SerializationCtx {}
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
