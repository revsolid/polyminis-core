use ncollide::world::{CollisionWorld, CollisionWorld2, CollisionGroups, CollisionObject2, GeometricQueryType};
use ncollide::shape::{Compound, Compound2, ShapeHandle2};
use nalgebra::{zero, Isometry2, Vector2};

use ::morphology::*;

#[derive(Clone)]
struct PolyminiPhysicsData;

// Physics
//
pub struct Physics
{
    uuid: usize,
    pos: Vector2<f32>,
    orientation: u8,
}
impl Physics
{
    // Private
    fn shapes_from_morphology(_: &Morphology) -> Compound2<f32>
    {
        // TODO: fill
        // First pass, shape = box of dimensions = Morph.dimensions
        Compound::new(vec![])
    }

    // Public
    pub fn new(uuid: usize, x: f32, y: f32, orientation: u8) -> Physics
    {
        Physics { uuid: uuid, pos: Vector2::new(x, y), orientation: orientation }
    }
 
    pub fn get_pos(&self) -> (f32, f32)
    {
        (self.pos.x, self.pos.y)
    }

    pub fn update(&mut self, physics_world: &PhysicsWorld)
    {
        let o = physics_world.get(self.uuid).unwrap();

        self.pos = o.position.translation;

        //TODO: Calculate orientation, go through any collision events
        // etc...
    }
}


// Physics World
pub struct PhysicsWorld
{
    c: CollisionWorld2<f32, PolyminiPhysicsData>
}
impl PhysicsWorld
{
    pub fn new() -> PhysicsWorld
    {
        PhysicsWorld { c: CollisionWorld::new(0.02, false) }
    }

    pub fn add(&mut self, physics: &Physics, morph: &Morphology)
    {
        let shapes = Physics::shapes_from_morphology(morph);

        //TODO: QueryType, CollisionGroups 
        self.c.deferred_add(physics.uuid, Isometry2::new(zero(), zero()), 
                            ShapeHandle2::new(shapes),
                            CollisionGroups::new(), GeometricQueryType::Proximity(0.0),
                            PolyminiPhysicsData{});
    }

    pub fn step(&mut self)
    {
        self.c.update();
    }

    fn get(&self, id: usize) -> Option<&CollisionObject2<f32, PolyminiPhysicsData>>
    {
        self.c.collision_object(id)
    }
}
