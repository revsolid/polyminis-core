use ::control::*;
use ::ph::*;
use ::physics::*;
use ::polymini::*;
use ::random::*;
use ::serialization::*;
use ::species::*;
use ::thermal::*;
use ::uuid::*;

use std::collections::HashSet;
use std::cmp::max;

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
    pub fn new_from_params(params :Vec<WorldObjectParams>) -> WorldObject
    {
        WorldObject
        {
            uuid: PolyminiUUIDCtx::next(),
            params: params,
        }
    }
    pub fn new_static_object( position: (f32, f32), dimensions: (u8, u8), permanent: bool) -> WorldObject
    {
        let mut params = vec![ WorldObjectParams::PhysicsWorldParams { position: position, dimensions: dimensions } ];

        if permanent
        {
            params.push(WorldObjectParams::PermanentWorldParams);
        }
        WorldObject::new_from_params(params)
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
                &WorldObjectParams::ThermoWorldParams { current_temperature: ct } =>
                {
                    json_obj.insert("Temperature".to_owned(), ct.to_json()); 
                },
                &WorldObjectParams::PhWorldParams { current_ph: cp } =>
                {
                    json_obj.insert("Ph".to_owned(), cp.to_json());
                },
                &WorldObjectParams::PermanentWorldParams =>
                {
                    json_obj.insert("Permanent".to_owned(), true.to_json());
                },
                &WorldObjectParams::BorderWorldParams =>
                {
                    json_obj.insert("IsBorder".to_owned(), true.to_json());
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

                if JsonUtils::verify_has_fields(json_obj, &vec!["Temperature".to_owned()])
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
    fn populate_world(dims: (f32, f32),
                      temp_range: (f32, f32),
                      ph_range: (f32, f32),
                      density: f32,
                      add_border: bool,
                      border_margin: f32,
                      rand_ctx: &mut PolyminiRandomCtx) -> Vec<WorldObject>
    {
        let mut objects = vec![];
        // Temperature
        let temp_min = temp_range.0;
        let temp_max = temp_range.1;
        
        // Ph
        let ph_min = ph_range.0;
        let ph_max = ph_range.1;
        
        // Density - How many dumb rocks, it's a percentage of total area (Max = 25%)
        let rho = density / 100.0;
        let mut obstacle_area_left = (rho * dims.0 * dims.1) as i32;
        
        // Surround it with walls? (on by default)
        if add_border 
        {
            let ubm = border_margin as u8;
            objects.push(WorldObject::new_border_object( (0.0, 0.0),   (dims.0 as u8, ubm + 1)));
            objects.push(WorldObject::new_border_object( (0.0, 0.0),   (ubm + 1, dims.1 as u8)));
            objects.push(WorldObject::new_border_object( (dims.0 - 1.0 - border_margin, 0.0 + border_margin),  (ubm + 1, dims.1 as u8)));
            objects.push(WorldObject::new_border_object( (0.0 + border_margin, dims.1 - 1.0 - border_margin),  (dims.0 as u8, ubm + 1)));
        }
        
        // Budget for Generators 
        //
        // The idea is to use simple 'Dice-like' algorithms 
        //
        // Obstacles -- TODO: Ideally the obstacles would also interact a bit with
        // temperature and PH
        while(obstacle_area_left > 0) 
        {
            // Create Obstacle and consume area left
            let side_x = max(1, rand_ctx.gen_range(0, 4) + rand_ctx.gen_range(0, 4));
            let side_y = max(1, rand_ctx.gen_range(0, 4) + rand_ctx.gen_range(0, 4));
            let pos_x  = rand_ctx.gen_range(0.0, dims.0);
            let pos_y  = rand_ctx.gen_range(0.0, dims.1);
        
            objects.push(WorldObject::new_static_object((pos_x, pos_y), (side_x, side_y), false));

            obstacle_area_left -= (side_x * side_y) as i32;
        }


        let calc_emmitter_v = |rand_ctx: &mut PolyminiRandomCtx,
                               dims: (f32, f32), min_v: f32, max_v: f32| -> f32
                               {
                                  // The idea is to make emitters fall mostly in the midpoint between the average
                                  // temperature (temp_mid_p) and the extremes (temp_min, temp_max).
                                  // We use 2 "dice" so that 0+0 is very rare (basically making a useless emitter
                                  // that drives temp to the average) and max_r + max_r is also rare (emitters
                                  // pulling to extreme temperatures)
                                  let value_delta_dir = if rand_ctx.test_value(0.5) { 1.0 } else { -1.0 } ;  

                                  let value_mid_p = min_v +  (max_v - min_v) / 2.0; 

                                  let range = ((max_v - value_mid_p) / 2.0).max(0.001);
                                  let emmitter_v = value_mid_p + (value_delta_dir * (rand_ctx.gen_range(0.0, range * 2.0)));
                                  debug!("PopulateWorld::CreateEmmitter Inputs:\n  Dims {:?}\n TempRange: {:?}\n PhRange: {:?}", dims, temp_range, ph_range);
                                  debug!("PopulateWorld::CreateEmmitter Value Mid Point: {:?} Value Delta Dir: {:?} Range: {:?}", value_mid_p, value_delta_dir, range); 
                                  debug!("PopulateWorld::CreateEmmitter Result: {:?}", emmitter_v); 
                                  emmitter_v
                               };
        
        // For now use a fixed number for Temp and PH Emitters 
        let num_emmitters = 4;
        for i in 0..num_emmitters
        {

            let temp_intensity = max(1, rand_ctx.gen_range(0, 3) + rand_ctx.gen_range(0, 3)); 
            let em_temp = calc_emmitter_v(rand_ctx, dims, temp_min, temp_max);
            let temp_pos_x  = rand_ctx.gen_range(0.0, dims.0);
            let temp_pos_y  = rand_ctx.gen_range(0.0, dims.1);
            objects.push(
                WorldObject::new_from_params(
                    vec![
                        WorldObjectParams::PhysicsWorldParams { position: (temp_pos_x, temp_pos_y), dimensions: (0, 0) },
                        WorldObjectParams::ThermoWorldParams { current_temperature: em_temp }
                    ]
                ));

            let ph_pos_x  = rand_ctx.gen_range(0.0, dims.0);
            let ph_pos_y  = rand_ctx.gen_range(0.0, dims.1);
            let ph_intensity = max(1, rand_ctx.gen_range(0, 3) + rand_ctx.gen_range(0, 3)); 
            let em_ph = calc_emmitter_v(rand_ctx, dims, ph_min, ph_max);
            objects.push(
                WorldObject::new_from_params(
                    vec![
                        WorldObjectParams::PhysicsWorldParams { position: (ph_pos_x, ph_pos_y), dimensions: (0, 0) },
                        WorldObjectParams::PhWorldParams { current_ph: em_ph}
                    ]
                ));
        }

        objects
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
    pub border_margin: f32,

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
            border_margin: 0.0,
        };
        env
    }

    pub fn new_from_json(json: &Json, rand_ctx: &mut PolyminiRandomCtx) -> Option<Environment>
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


                let mut temp_max = 1.0;
                let mut temp_min = 0.0;
                let tworld = match json_obj.get("Temperature")
                {
                    Some(&Json::Object(ref temp_obj)) =>
                    {
                        temp_max = temp_obj.get("Max").unwrap().as_f64().unwrap() as f32;
                        temp_min = temp_obj.get("Min").unwrap().as_f64().unwrap() as f32;
                        ThermoWorld::new_with_dimensions(dims, (temp_min+temp_max)/2.0)
                    },
                    _ =>
                    {
                        ThermoWorld::new_with_dimensions(dims, 0.5)
                    }
                };


                let mut ph_max = 1.0;
                let mut ph_min = 0.0;
                let phworld = match json_obj.get("Ph")
                {
                    Some(&Json::Object(ref temp_obj)) =>
                    {
                        ph_max = temp_obj.get("Max").unwrap().as_f64().unwrap() as f32;
                        ph_min = temp_obj.get("Min").unwrap().as_f64().unwrap() as f32;
                        PhWorld::new_with_dimensions(dims, (ph_min+ph_max)/2.0)
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
                        rho as f32
                    },
                    _ =>
                    {
                        0.5 // TODO: Some Default?
                    }
                };

                let margin = json_obj.get("BorderMargin").unwrap_or(&Json::F64(0.0)).as_f64().unwrap() as f32;
                let mut env = Environment {
                              dimensions: dims,
                              physical_world: PhysicsWorld::new_with_dimensions(dims),
                              thermal_world: tworld,
                              ph_world:  phworld,
                              density: density,
                              default_sensors: default_sensors,
                              species_slots: json_obj.get("SpeciesSlots").unwrap().as_u64().unwrap() as usize,
                              objects: vec![],
                              permanent_objects: HashSet::new(),
                              border_margin: margin
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


                let add_border = json_obj.get("AddBorder").unwrap_or(&Json::Boolean(true)).as_boolean().unwrap();
                /*{
                    env.add_object(WorldObject::new_border_object( (0.0, 0.0),   (dims.0 as u8, 1)));
                    env.add_object(WorldObject::new_border_object( (0.0, 0.0),   (1, dims.1 as u8)));
                    env.add_object(WorldObject::new_border_object( (dims.0 - 1.0, 0.0),  (1, dims.1 as u8)));
                    env.add_object(WorldObject::new_border_object( (0.0, dims.1 - 1.0),  (dims.0 as u8, 1)));
                }*/


                // Populate World
                if json_obj.get("UseWorldBuilder").unwrap_or(&Json::Boolean(false)).as_boolean().unwrap()
                {
                    let mut objs = WorldBuilder::populate_world(dims, (temp_min, temp_max), (ph_min, ph_max), density, add_border, margin, rand_ctx);
                    let l = objs.len();
                    for o in objs.drain(0..l)
                    {
                        env.add_object(o);
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
                    self.thermal_world.add_object(world_object.uuid, pos, ct, 5.0);
                },
                WorldObjectParams::PhWorldParams { current_ph: cph } => 
                {
                    self.ph_world.add_object(world_object.uuid, pos, cph, 5.0);
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
            let mut all_obj_json_arr = pmJsonArray::new();
            for obj in &self.objects
            {
                if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB) && self.permanent_objects.contains(&obj.uuid)
                {
                    perm_obj_json_arr.push(obj.serialize(ctx));
                }

                if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC) && !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
                {
                    all_obj_json_arr.push(obj.serialize(ctx)); 
                }
            }

            if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
            {
                json_obj.insert("PermanentObjects".to_owned(), Json::Array(perm_obj_json_arr));
            }
            else
            {
                json_obj.insert("WorldObjects".to_owned(), Json::Array(all_obj_json_arr));
            }
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC) &&
          !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DB)
        {
            //
            json_obj.insert("PhysicsWorld".to_owned(), self.physical_world.serialize(ctx));

            //
            json_obj.insert("ThermalWorld".to_owned(), self.thermal_world.serialize(ctx));

            //
            json_obj.insert("PhWorld".to_owned(), self.ph_world.serialize(ctx));
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
