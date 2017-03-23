use ::control::*;
use ::ph::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
use ::thermal::*;
use ::uuid::*;

const KENVIRONMENT_DIMENSIONS: (f32, f32) = (50.0, 50.0);

// NOTE: Stubbing out what should be the World / Object hierarchy
#[derive(Clone, Copy)]
pub enum WorldObjectParams
{
    PhysicsWorldParams { position: (f32, f32), dimensions: (u8, u8) },
    ThermoWorldParams { position: (f32, f32), currentTemperature: f32, emmitIntensity: f32 },
    PhWorldParams { position: (f32, f32), currentTemperature: f32, emmitIntensity: f32 },
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
    pub fn new_static_object( position: (f32, f32), dimensions: (u8, u8)) -> WorldObject
    {
        WorldObject
        {
            uuid: PolyminiUUIDCtx::next(),
            params: vec![ WorldObjectParams::PhysicsWorldParams { position: position, dimensions: dimensions } ],
        }
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
}
impl Environment
{
    pub fn new(species_slots: usize, default_sensors: Vec<Sensor>) -> Environment
    {
        let dimensions = KENVIRONMENT_DIMENSIONS;
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
                            };

                //TODO: This is temporary
                env.add_static_object( (0.0, 0.0),   (dims.0 as u8, 1));
                env.add_static_object( (0.0, 0.0),   (1, dims.1 as u8));
                env.add_static_object( (dims.0 - 1.0, 0.0),  (1, dims.1 as u8));
                env.add_static_object( (0.0, dims.1 - 1.0),  (dims.0 as u8, 1));

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
            true
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
        for p in &world_object.params
        {
            match *p
            {
                WorldObjectParams::PhysicsWorldParams { position: p, dimensions: d } =>
                {
                    self.physical_world.add_object(world_object.uuid, p, d);
                },
                _ => {},
            }
        }

        self.objects.push(world_object);
    }

    pub fn add_static_object(&mut self, position: (f32, f32), dimensions: (u8, u8))
    {
        self.add_object(WorldObject::new_static_object(position, dimensions));
    }

    pub fn get_species_slots(&self) -> usize
    {
        self.species_slots
    }

    pub fn advance_epoch(&self) -> Environment
    {
        let mut to_ret = Environment::new(self.species_slots,
                                          self.default_sensors.clone());

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
            let json_arr: pmJsonArray = self.default_sensors.iter().map(|s| { s.tag.serialize(ctx) }).collect();
            json_obj.insert("DefaultSensors".to_owned(), Json::Array(json_arr));

            //
            let mut dimensions_json = pmJsonObject::new();
            dimensions_json.insert("x".to_owned(), self.dimensions.0.to_json());
            dimensions_json.insert("y".to_owned(), self.dimensions.1.to_json());
            json_obj.insert("Dimensions".to_owned(), Json::Object(dimensions_json));

            //
            json_obj.insert("SpeciesSlots".to_owned(), self.species_slots.to_json());
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
        let mut to_ret = Environment::new(self.species_slots,
                                          self.default_sensors.clone());

        for o in &self.objects
        {
            to_ret.add_object(o.clone());
        }

        to_ret

    }
}
