use ::control::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
use ::uuid::*;

const KENVIRONMENT_DIMENSIONS: (f32, f32) = (100.0, 100.0);

// NOTE: Stubbing out what should be the World / Object hierarchy
#[derive(Clone, Copy)]
pub enum WorldObjectParams
{
    PhysicsWorldParams { position: (f32, f32), dimensions: (u8, u8) },
    TemperatureWorldParams { temperature: f32 },
    // ETC..
}
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


pub struct Environment
{
    pub dimensions: (f32, f32),
    pub physical_world: PhysicsWorld,
    pub objects: Vec<WorldObject>,
    pub default_sensors: Vec<Sensor>, 
    species_slots: usize,
}
impl Environment
{
    pub fn new(species_slots: usize, default_sensors: Vec<Sensor>) -> Environment
    {
        let dimensions = KENVIRONMENT_DIMENSIONS;
        let mut env = Environment
        {
            dimensions: dimensions,
            physical_world: PhysicsWorld::new(),
            default_sensors: default_sensors,
            species_slots: species_slots,
            objects: vec![],
        };

        env.add_static_object( (0.0, 0.0), (dimensions.0 as u8, 1));
        env.add_static_object( (0.0, 0.0), (1, dimensions.1 as u8));
        env.add_static_object( (dimensions.0 - 1.0, 0.0), (1, dimensions.1 as u8));
        env.add_static_object( (0.0, dimensions.1 - 1.0), (dimensions.0 as u8, 1));

        env
    }

    pub fn add_individual(&mut self, polymini: &mut Polymini) -> bool 
    {
        let mut res = false;
        res = self.physical_world.add(polymini.get_physics_mut());

        res
        //TODO: Add to other worlds
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
        //let uuid = PolyminiUUIDCtx::next();
        //self.physical_world.add_object(uuid, position, dimensions);
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
}
impl Serializable for Environment
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            //
            json_obj.insert("PhysicsWorld".to_owned(), self.physical_world.serialize(ctx));
            
        }
        Json::Object(json_obj)
    }
}
