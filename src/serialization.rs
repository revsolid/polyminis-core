//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE serialize 
extern crate rustc_serialize;
pub use self::rustc_serialize::json::{self, Json, ToJson};
//
//

pub type pmJsonObject = json::Object;
pub type pmJsonArray = json::Array;

pub mod PolyminiSerializationFlags
{
    bitflags!
    {
        pub flags SerializationFlags: u32
        {
            const PM_SF_NONE    = 0b00000000,

            const PM_SF_STATIC  = 0b00000001,

            const PM_SF_DYNAMIC = 0b00000010,

            const PM_SF_DB_INT  = 0b00000100,

            const PM_SF_STATS   = 0b00001000,

            const PM_SF_DEBUG   = ( PM_SF_STATIC.bits | PM_SF_DYNAMIC.bits | PM_SF_DB.bits | PM_SF_STATS.bits ),

            const PM_SF_DB      = ( PM_SF_STATIC.bits | PM_SF_DB_INT.bits ),
        }
    }
}
use self::PolyminiSerializationFlags::*;

pub struct SerializationCtx
{
    flags: SerializationFlags,
}
impl SerializationCtx
{
    pub fn new() -> SerializationCtx
    {
        SerializationCtx { flags: PM_SF_NONE }
    }

    pub fn new_from_flags(flags: SerializationFlags) -> SerializationCtx
    {
        SerializationCtx { flags: flags }
    }
    
    pub fn debug() -> SerializationCtx
    {
        SerializationCtx { flags: PM_SF_DEBUG }
    }

    pub fn has_flag(&self, flags: SerializationFlags) -> bool
    {
        self.flags.contains(flags)
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

pub trait Deserializable
{
    fn new_from_json(json: &Json, ctx: &mut SerializationCtx) -> Option<Self> where Self: Sized;
    fn update_from(&mut self, json: &Json, ctx: &mut SerializationCtx)
    {
        // Should log something to remind the caller that this is empty
    }
}
