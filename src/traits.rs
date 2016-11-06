use std::fmt;
use std::mem::transmute;

use ::actuators::*;
use ::sensors::*;
use ::serialization::*;

//
//
pub const TIER_ONE_TO_TWO_CHAIN:   u8 = 0xFF;
pub const TIER_TWO_TO_THREE_CHAIN:   u8 = 0x0F;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TraitTier
{
    TierI,
    TierII,
    TierIII,
}
impl From<u8> for TraitTier
{
    fn from(v: u8) -> TraitTier
    {
        match v
        {
            1 => { TraitTier::TierI },
            2 => { TraitTier::TierII },
            3 => { TraitTier::TierIII },
            default => { panic!("Tier too damn high") }
        }
    }
}
impl fmt::Display for TraitTier 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}
impl Serializable for TraitTier
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        self.to_string().to_json()
    }
}

//
#[derive(Debug, Clone, Copy)]
pub struct Trait
{
    pub trait_tier: TraitTier,
    pub trait_number: u8,
    pub pm_trait: PolyminiTrait 
}
impl Trait
{
    pub fn new(tier: TraitTier, trait_num: u8, pm_trait: PolyminiTrait) -> Trait
    {
        Trait
        {
            trait_tier: tier,
            trait_number: trait_num,
            pm_trait: pm_trait
        }
    }
}
impl Serializable for Trait
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("trait_tier".to_string(), self.trait_tier.serialize(ctx));
        json_obj.insert("organelle_id".to_string(), self.trait_number.to_json());
        Json::Object(json_obj)
    }
}

//
#[derive(Debug, Clone, Copy)]
pub enum PolyminiSimpleTrait
{
    Empty,
    SpeedTrait,
}

#[derive(Debug, Clone, Copy)]
pub enum PolyminiTrait
{
    PolyminiActuator(ActuatorTag),
    PolyminiSensor(SensorTag),
    PolyminiSimpleTrait(PolyminiSimpleTrait)
}
