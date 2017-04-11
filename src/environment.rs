use ::control::*;
use ::ph::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
use ::thermal::*;
use ::uuid::*;

use std::collections::HashSet;

const KENVIRONMENT_DIMENSIONS: (f32, f32) = (50.0, 50.0);

// NOTE: Stubbing out what should be the World / Object hierarchy
#[derive(Clone, Copy)]
pub enum WorldObjectParams
{
    PhysicsWorldParams { position: (f32, f32), dimensions: (u8, u8) },
    ThermoWorldParams  { current_temperature: f32 },
    PhWorldParams      { current_ph: f32 },

    // Objects with this Params get Serialized / Written in the DB
    PermanentWorldParams,

    // If this object is part of a 'border'
    BorderWorldParams, // Might be getting a bit ridic.

    // ETC..
}
#[derive(Clone)]
pub struct WorldObject
{
    uuid: PUUID,
    params: Vec<WorldObjectParams>,
}
impl WorldObject
{
    pub fn new_static_object( position: (f32, f32), dimensions: (u8, u8), permanent: bool) -> WorldObject
    {
        let mut params = vec![ WorldObjectParams::PhysicsWorldParams { position: position, dimensions: dimensions } ];

        if permanent
        {
            params.push(WorldObjectParams::PermanentWorldParams);
        }

        WorldObject
        {
            uuid: PolyminiUUIDCtx::next(),
            params: params,
        }
    }

    pub fn new_border_object(position: (f32, f32), dimensions: (u8, u8)) -> WorldObject
    {
        let mut wo = WorldObject::new_static_object(position, dimensions, false);
        wo.params.push(WorldObjectParams::BorderWorldParams);
        wo
    }

    pub fn advance_epoch(&self) -> WorldObject
    {
        // TODO: Different WorldObjects should be free to advance_epoch in different ways,
        // but a context might be required (What information should Objects require to advance?)
        // For now just create a new object with the same params.
        WorldObject
        {
            uuid: PolyminiUUIDCtx::next(),
            params: self.params.clone()
        }
    }
}
impl Serializable for WorldObject
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        for p in &self.params
        {
            match p
            {
                &WorldObjectParams::PhysicsWorldParams { position: p, dimensions: d }  =>
                {
                    json_obj.insert("Position".to_owned(), Vector2::new(p.0, p.1).serialize(ctx));
                    json_obj.insert("Dimensions".to_owned(), Vector2::new(d.0 as f32, d.1 as f32).serialize(ctx));
                },
                &WorldObjectParams::ThermoWorldParams { current_temperature: _ } =>
                {
                },
                &WorldObjectParams::PhWorldParams { current_ph: _ } =>
                {
                },
                &WorldObjectParams::PermanentWorldParams =>
                {
                },
                &WorldObjectParams::BorderWorldParams =>
                {
                },
                _ =>
                {
                    // TODO: Other Params
                }
            }
        }
        Json::Object(json_obj)
    }
}
impl Deserializable for WorldObject
{
    fn new_from_json(json: &Json, ctx: &mut SerializationCtx) -> Option<WorldObject> 
    {
        debug!("Env::WorldObject::Serialization - {}", json.to_string()); 
        match json
        {
            &Json::Object(ref json_obj) =>
            {
                let mut params = vec![];
                if JsonUtils::verify_has_fields(json_obj, &vec!["Position".to_owned(), "Dimensions".to_owned()])
                {
                    let pos =  {
                        let p = json_obj.get("Position").unwrap().as_object().unwrap();
                        (p.get("x").unwrap().as_f64().unwrap() as f32,
                         p.get("y").unwrap().as_f64().unwrap() as f32)
                    };

                    let dims =  {
                        let d = json_obj.get("Dimensions").unwrap().as_object().unwrap();
                        (d.get("x").unwrap().as_f64().unwrap() as u8,
                         d.get("y").unwrap().as_f64().unwrap() as u8)
                    };

                    debug!("Env::WorldObject::Serialization adding Physics Params"); 
                    params.push( WorldObjectParams::PhysicsWorldParams {
                                    position: pos,   
                                    dimensions: dims,   
                                 });
                }

                if JsonUtils::verify_has_fields(json_obj, &vec!["Position".to_owned(), "Dimensions".to_owned()])
                {
                }

                debug!("Env::WorldObject::Serialization Params Len - {}", params.len()); 
                if params.len() == 0
                {
                    None
                }
                else
                {
                    Some(WorldObject
                        {
                            params: params,
                            uuid: PolyminiUUIDCtx::next(),
                        })
                }
            },


            _ =>
            {
                None
            }
        }
    }
}

// *
//
// trait World (?)
// {
//   Add(WorldObject);
// }
//
// *//

// ~NOTE
//
struct WorldBuilder;
impl WorldBuilder
{
    // Json goes in -
    // WorldObjects come out
    fn populate_world(json_world: &Json) -> Vec<WorldObject>
    {
        match *json_world
        {
            Json::Object(ref world_config) => 
            {
                // Temperature
                
                // Ph
                
                // Density - How many dumb rocks
                
                // Surround it with walls? (on by default)
                
                // Material / Comp Information 
                
                // Budget for Generators 
                vec![]
            },
            _ =>
            {
                // Is there a sensible default?
                vec![]
            }
        }

    }
}


pub struct Environment
{
    // 
    pub dimensions: (f32, f32),
    pub density: f32,
    pub default_sensors: Vec<Sensor>, 
    pub species_slots: usize,

    // Worlds
    pub physical_world: PhysicsWorld,
    pub thermal_world: ThermoWorld,
    pub ph_world: PhWorld,

    //
    pub objects: Vec<WorldObject>,

    // 
    permanent_objects: HashSet<PUUID>,
}
impl Environment
{
    pub fn new(species_slots: usize, default_sensors: Vec<Sensor>) -> Environment
    {
        let dimensions = KENVIRONMENT_DIMENSIONS;
        Environment::new_with_dimensions(species_slots, default_sensors, dimensions)
    }

    pub fn new_with_dimensions(species_slots: usize, default_sensors: Vec<Sensor>, dimensions: (f32, f32)) -> Environment
    {
        let mut env = Environment
        {
            dimensions: dimensions,
            density: 0.5,
            physical_world: PhysicsWorld::new_with_dimensions(dimensions),
            thermal_world: ThermoWorld::new_with_dimensions(dimensions, 0.5),
            ph_world: PhWorld::new_with_dimensions(dimensions, 0.5),
            default_sensors: default_sensors,
            species_slots: species_slots,
            objects: vec![],
            permanent_objects: HashSet::new(),
        };
        env
    }

    pub fn new_from_json(json: &Json) -> Option<Environment>
    {
        match *json
        {
            Json::Object(ref json_obj) =>
            {
                let default_sensors = json_obj.get("DefaultSensors").unwrap().as_array().unwrap().iter().map( 
                |s_tag|
                {
                    Sensor::new(SensorTag::new_from_json(s_tag, &mut SerializationCtx::new()).unwrap(), 0)
                }).collect();

                let dims =  {
                    let d = json_obj.get("Dimensions").unwrap().as_object().unwrap();
                    (d.get("x").unwrap().as_f64().unwrap() as f32,
                     d.get("y").unwrap().as_f64().unwrap() as f32)
                };


                let tworld = match json_obj.get("Temperature")
                {
                    Some(&Json::Object(ref temp_obj)) =>
                    {
                        let max = temp_obj.get("Max").unwrap().as_f64().unwrap();
                        let min = temp_obj.get("Min").unwrap().as_f64().unwrap();
                        ThermoWorld::new_with_dimensions(dims, ((min+max)/2.0) as f32)
                    },
                    _ =>
                    {
                        ThermoWorld::new_with_dimensions(dims, 0.5)
                    }
                };


                let phworld = match json_obj.get("Ph")
                {
                    Some(&Json::Object(ref temp_obj)) =>
                    {
                        let max = temp_obj.get("Max").unwrap().as_f64().unwrap();
                        let min = temp_obj.get("Min").unwrap().as_f64().unwrap();
                        PhWorld::new_with_dimensions(dims, ((min+max)/2.0) as f32)
                    },
                    _ =>
                    {
                        PhWorld::new_with_dimensions(dims, 0.5)
                    }
                };

                let density = match json_obj.get("Density")
                {
                    Some(&Json::F64(rho)) =>
                    {
                        rho
                    },
                    _ =>
                    {
                        0.5 // TODO: Some Default?
                    }
                };

                let mut env = Environment {
                              dimensions: dims,
                              physical_world: PhysicsWorld::new_with_dimensions(dims),
                              thermal_world: tworld,
                              ph_world:  phworld,
                              density: density as f32,
                              default_sensors: default_sensors,
                              species_slots: json_obj.get("SpeciesSlots").unwrap().as_u64().unwrap() as usize,
                              objects: vec![],
                              permanent_objects: HashSet::new(),
                            };

                
                match json_obj.get("PermanentObjects")
                {
                    Some(&Json::Array(ref objects)) =>
                    {
                        for o in objects
                        {
                            match WorldObject::new_from_json(o, &mut SerializationCtx::new())
                            {
                                Some(w_obj) =>
                                {

                                    debug!("Env::WorldObject::AddingPermanent Object to Env"); 
                                    env.add_object(w_obj);
                                }
                                _ =>
                                {
                                    debug!("Env::WorldObject:: Could NOT deserialize Permanent Object");
                                }
                            }
                        }
                    },
                    _ => {}
                }


                if json_obj.get("AddBorder").unwrap_or(&Json::Null).as_boolean().unwrap_or(false)
                {
                    env.add_object(WorldObject::new_border_object( (0.0, 0.0),   (dims.0 as u8, 1)));
                    env.add_object(WorldObject::new_border_object( (0.0, 0.0),   (1, dims.1 as u8)));
                    env.add_object(WorldObject::new_border_object( (dims.0 - 1.0, 0.0),  (1, dims.1 as u8)));
                    env.add_object(WorldObject::new_border_object( (0.0, dims.1 - 1.0),  (dims.0 as u8, 1)));
                }


                {
                    let cfg = json_obj.get("WorldConfig");
                    if cfg.is_some() 
                    {
                        let mut objs = WorldBuilder::populate_world(cfg.unwrap());
                        let l = objs.len();
                        for o in objs.drain(0..l)
                        {
                            env.add_object(o);
                        }
                    }
                }
                
                Some(env)
            },
            _ => 
            {
                None
            }
        }
    }

    pub fn add_individual(&mut self, polymini: &mut Polymini) -> bool 
    {
        let mut res = false;
        res = self.physical_world.add(polymini.get_physics_mut());

        if (!res)
        {
            false
        }
        else
        {
            let pos = polymini.get_physics().get_starting_pos();
            res &= self.thermal_world.add(polymini.get_thermo_mut(), pos);
            res &= self.ph_world.add(polymini.get_ph_mut(), pos);
            res 
        }
    }

    pub fn add_individual_force_pos(&mut self, polymini: &mut Polymini) -> bool
    {
        let pos = polymini.get_physics().get_pos();

        self.add_individual(polymini) && {
            let n_spos = polymini.get_physics().get_starting_pos();
            let dx = (pos.0 - n_spos.0).abs();
            let dy = (pos.1 - n_spos.1).abs();
            (dx <= 3.0 && dy <= 3.0)
        }
    }

    pub fn remove_individual(&mut self, polymini: &mut Polymini) -> bool
    {
        let mut res = false;
        res = self.physical_world.remove(polymini.get_physics_mut());
        res 
        //TODO: Remove from other worlds
    }

    pub fn add_object(&mut self, world_object: WorldObject)
    {
        let mut pos = (0.0, 0.0);
        let mut dims = (0, 0);


        // First Pass - Physics and standalone params 
        for params in &world_object.params
        {
            match *params
            {
                WorldObjectParams::PhysicsWorldParams { position: p, dimensions: d } =>
                {
                    self.physical_world.add_object(world_object.uuid, p, d);
                    pos = p;
                    dims = d;
                },
                WorldObjectParams::PermanentWorldParams =>
                {
                    self.permanent_objects.insert(world_object.uuid);
                },
                _ => {},
            }
        }

        // Second Pass - Everything That Depends on Placement 
        // If position and dims == (0,0) that's ok better than crashing or
        // having a complex dependency system
        for p in &world_object.params 
        {
            match *p
            {
                WorldObjectParams::ThermoWorldParams { current_temperature: ct } => 
                {
                    self.thermal_world.add_object(world_object.uuid, pos, ct, 1.0);
                },
                WorldObjectParams::PhWorldParams { current_ph: cph } => 
                {
                    self.thermal_world.add_object(world_object.uuid, pos, cph, 1.0);
                },
                _ => {},
            }
        }

        self.objects.push(world_object);
    }

    pub fn add_static_object(&mut self, position: (f32, f32), dimensions: (u8, u8),
                             permanent: bool)
    {
        self.add_object(WorldObject::new_static_object(position, dimensions, permanent));
    }

    pub fn get_species_slots(&self) -> usize
    {
        self.species_slots
    }

    pub fn advance_epoch(&self) -> Environment
    {
        let mut to_ret = Environment::new_with_dimensions(self.species_slots,
                                          self.default_sensors.clone(),
                                          self.dimensions);

        for o in &self.objects
        {
            to_ret.add_object(o.advance_epoch());
        }

        to_ret
    }

    pub fn restart(&self) -> Environment
    {
        // For now advance_epoch and restart are equivalent
        self.advance_epoch()
    }
}
impl Serializable for Environment
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            //
            let sensor_json_arr: pmJsonArray = self.default_sensors.iter().map(|s| { s.tag.serialize(ctx) }).collect();
            json_obj.insert("DefaultSensors".to_owned(), Json::Array(sensor_json_arr));

            //
            let mut dimensions_json = pmJsonObject::new();
            dimensions_json.insert("x".to_owned(), self.dimensions.0.to_json());
            dimensions_json.insert("y".to_owned(), self.dimensions.1.to_json());
            json_obj.insert("Dimensions".to_owned(), Json::Object(dimensions_json));

            //
            json_obj.insert("SpeciesSlots".to_owned(), self.species_slots.to_json());

            //
            let mut perm_obj_json_arr = pmJsonArray::new();
            for obj in &self.objects
            {
                if self.permanent_objects.contains(&obj.uuid)
                {
                    perm_obj_json_arr.push(obj.serialize(ctx));
                }
            }
            json_obj.insert("PermanentObjects".to_owned(), Json::Array(perm_obj_json_arr));
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC) &&
          !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            //
            json_obj.insert("PhysicsWorld".to_owned(), self.physical_world.serialize(ctx));
        }
        Json::Object(json_obj)
    }
}
impl Clone for Environment
{
    fn clone(&self) -> Environment
    {
        let mut to_ret = Environment::new_with_dimensions(self.species_slots,
                                                          self.default_sensors.clone(),
                                                          self.dimensions);

        for o in &self.objects
        {
            to_ret.add_object(o.clone());
        }

        to_ret

    }
}
