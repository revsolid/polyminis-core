use ::control::*;
use ::physics::*;
use ::polymini::*;
use ::serialization::*;
use ::species::*;
use ::uuid::*;

const KENVIRONMENT_DIMENSIONS: (f32, f32) = (100.0, 100.0);
pub struct Environment
{
    pub dimensions: (f32, f32),
    pub physical_world: PhysicsWorld,
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
            species_slots: species_slots
        };

        env.add_object( (0.0, 0.0), (dimensions.0 as u8, 1));
        env.add_object( (0.0, 0.0), (1, dimensions.1 as u8));
        env.add_object( (dimensions.0 - 1.0, 0.0), (1, dimensions.1 as u8));
        env.add_object( (0.0, dimensions.1 - 1.0), (dimensions.0 as u8, 1));

        env
    }

    pub fn add_individual(&mut self, polymini: &Polymini)
    {
        self.physical_world.add(polymini.get_physics());
        //TODO: Add to other worlds
    }

    pub fn add_object(&mut self, position: (f32, f32), dimensions: (u8, u8))
    {
        let uuid = PolyminiUUIDCtx::next();

        self.physical_world.add_object(uuid, position, dimensions);
        //TODO: Maybe add to other worlds
    }

    pub fn get_species_slots(&self) -> usize
    {
        self.species_slots
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
