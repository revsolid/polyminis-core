use std::fmt;

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
            _ => { panic!("Tier too damn high") }
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
impl Deserializable for TraitTier
{
    fn new_from_json(json: &Json, ctx: &mut SerializationCtx) -> Option<TraitTier>
    {
        let to_ret = match *json
        {
            Json::String(ref string_value) => 
            {
                if *string_value == TraitTier::TierI.to_string()
                {
                    Some(TraitTier::TierI)
                }
                else if *string_value == TraitTier::TierII.to_string()
                {
                    Some(TraitTier::TierII)
                }
                else if *string_value == TraitTier::TierIII.to_string()
                {
                    Some(TraitTier::TierIII)
                }
                else
                {
                    None
                }
            },
            _ =>
            {
                None
            }
        };
        to_ret
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
        json_obj.insert("Tier".to_string(), self.trait_tier.serialize(ctx));
        json_obj.insert("TID".to_string(), self.trait_number.to_json());
        Json::Object(json_obj)
    }
}
impl Deserializable for Trait
{
    fn new_from_json(json: &Json, ctx: &mut SerializationCtx) -> Option<Trait>
    {
        let trait_tier;
        let trait_number;

        if !json.is_object()
        {
            return None 
        }

        let json_obj = json.as_object().unwrap();

        match json_obj.get("Tier") 
        {
            Some(tier) =>
            {
                trait_tier = TraitTier::new_from_json(&tier, ctx).unwrap();
            },
            None => { return None }
        }

        match json_obj.get("TID")
        {
            Some(num) =>
            {
                trait_number = num.as_u64().unwrap_or(0) as u8;
            },
            None => { return None }
        }

        Some(Trait::new(trait_tier, trait_number,
                            PolyminiTrait::new_from_json(json_obj.get("Trait").unwrap(), ctx)
                            .unwrap_or(PolyminiTrait::PolyminiSimpleTrait(TraitTag::Empty))))
    }
}

//
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TraitTag
{
    Empty,
    SpeedTrait,
}
impl Deserializable for TraitTag
{
    fn new_from_json(json: &Json, _: &mut SerializationCtx) -> Option<TraitTag>
    {
        match *json
        {
            Json::String(ref json_string) =>
            {
                match json_string.as_ref()
                {
                    "speedtrait" => { Some(TraitTag::SpeedTrait) },
                    _ =>
                    {
                        None
                    }
                }
            }
            _ =>
            {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PolyminiTrait
{
    PolyminiActuator(ActuatorTag),
    PolyminiSensor(SensorTag),
    PolyminiSimpleTrait(TraitTag)
}
impl Deserializable for PolyminiTrait
{
    fn new_from_json(json: &Json, ctx:&mut SerializationCtx) -> Option<PolyminiTrait>
    {
        let actuator  = ActuatorTag::new_from_json(json, ctx);
        let sensor    = SensorTag::new_from_json(json, ctx);
        let pm_trait  = TraitTag::new_from_json(json, ctx);
        if actuator.is_some()
        {
            Some(PolyminiTrait::PolyminiActuator(actuator.unwrap()))
        }
        else if sensor.is_some()
        {
            Some(PolyminiTrait::PolyminiSensor(sensor.unwrap()))
        }
        else
        {
            Some(PolyminiTrait::PolyminiSimpleTrait(pm_trait.unwrap_or(TraitTag::Empty)))
        }
    }
}

#[cfg(test)]
mod test
{
    use super::*;
    use ::serialization::*;

    #[test]
    fn test_trait_deserialize()
    {
        let mut s_ctx = SerializationCtx::new();

        let t1 = "TierI";
        let t1_json = t1.to_string().to_json();
        let t1_p = TraitTier::new_from_json(&t1_json, &mut s_ctx);
        assert_eq!(t1_p.unwrap(), TraitTier::TierI);

        let t2 = "TierII";
        let t2_json = t2.to_string().to_json();
        let t2_p = TraitTier::new_from_json(&t2_json, &mut s_ctx);
        assert_eq!(t2_p.unwrap(), TraitTier::TierII);

        let t3 = "TierIII";
        let t3_json = t3.to_string().to_json();
        let t3_p = TraitTier::new_from_json(&t3_json, &mut s_ctx);
        assert_eq!(t3_p.unwrap(), TraitTier::TierIII);
    }

    // #[test]
    // test_trait_serialize() ?
    // If the previous test and the next tess pass there's no extra coverage
    // provided by a serialize only test

    #[test]
    fn test_trait_serialize_deserialize()
    {
        let mut s_ctx = SerializationCtx::new();

        let t1 = TraitTier::TierI;
        let t1_json = t1.serialize(&mut s_ctx);
        let t1_p = TraitTier::new_from_json(&t1_json, &mut s_ctx);
        assert_eq!(t1, t1_p.unwrap());

        let t2 = TraitTier::TierII;
        let t2_json = t2.serialize(&mut s_ctx);
        let t2_p = TraitTier::new_from_json(&t2_json, &mut s_ctx);
        assert_eq!(t2, t2_p.unwrap());

        let t3 = TraitTier::TierIII;
        let t3_json = t3.serialize(&mut s_ctx);
        let t3_p = TraitTier::new_from_json(&t3_json, &mut s_ctx);
        assert_eq!(t3, t3_p.unwrap());
    }
}
