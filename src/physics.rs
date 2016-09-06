//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE ncollide / nalgebra
extern crate nalgebra;
extern crate ncollide;

use self::nalgebra::{zero, Isometry2, Vector2, Point2};

use self::ncollide::narrow_phase::{ProximityHandler};
use self::ncollide::query::{Proximity};
use self::ncollide::shape::{Compound, Compound2, ShapeHandle2};
use self::ncollide::world::{CollisionWorld, CollisionWorld2,
                            CollisionGroups, CollisionObject2, GeometricQueryType};
//
//

use ::morphology::*;
use ::control::*;

#[derive(Clone)]
struct PolyminiPhysicsData;

pub type PhysicsAction = Action;

// Physics
//
pub struct Physics
{
    uuid: usize,
    pos: Vector2<f32>,
    orientation: u8,
    move_succeded: bool,
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
        Physics { uuid: uuid, pos: Vector2::new(x, y), orientation: orientation,
                  move_succeded: true }
    }

    pub fn get_pos(&self) -> (f32, f32)
    {
        (self.pos.x, self.pos.y)
    }

    pub fn get_orientation(&self) -> u8
    {
        self.orientation
    }

    pub fn get_move_succeded(&self) -> bool
    {
        self.move_succeded
    }

    // Attempt to add rotation / translation to our physics object
    pub fn act_on(&mut self, actions: ActionList, physics_world: &mut PhysicsWorld)
    {
        // Accumulate the actions into one result
        physics_world.apply(self.uuid, &actions[0]);
    }

    // Update our information from the result of the simulation
    pub fn update_state(&mut self, physics_world: &PhysicsWorld)
    {
        let o = physics_world.get(self.uuid).unwrap();

        self.pos = o.position.translation;

        //TODO: Calculate orientation, go through any collision events
        // etc...
    }
}

// Collision Handler
struct PhysicsWorldCollisionHandler;

impl ProximityHandler<Point2<f32>, Isometry2<f32>, PolyminiPhysicsData> for PhysicsWorldCollisionHandler
{
    fn handle_proximity(&mut self,
                        _: &CollisionObject2<f32, PolyminiPhysicsData>,
                        _: &CollisionObject2<f32, PolyminiPhysicsData>,
                        _: Proximity,
                        new_proximity: Proximity)
    {
        if new_proximity ==  Proximity::Intersecting
        {
            //
            // Touching
        }
    }
}


// Physics World
pub struct PhysicsWorld
{
    c: CollisionWorld2<f32, PolyminiPhysicsData>,

    //
    polyminis_cgroup: CollisionGroups,
    objects_cgroup: CollisionGroups
}
impl PhysicsWorld
{
    pub fn new() -> PhysicsWorld
    {
        let mut col_w = CollisionWorld::new(0.02, false);

        col_w.register_proximity_handler("phyisics_world_collision", PhysicsWorldCollisionHandler);

        let mut pcg = CollisionGroups::new();
        pcg.set_membership(&[1]);

        let mut ocg = CollisionGroups::new();
        ocg.set_membership(&[2]);
        ocg.set_whitelist(&[1]);

        let ph_w = PhysicsWorld { c: col_w,
                                  polyminis_cgroup: pcg,
                                  objects_cgroup: ocg
        };
        ph_w
    }

    pub fn add_object(&mut self)
    {
    }

    pub fn add(&mut self, physics: &Physics, morph: &Morphology)
    {
        let shapes = Physics::shapes_from_morphology(morph);

        //TODO: QueryType, CollisionGroups
        self.c.deferred_add(physics.uuid, Isometry2::new(zero(), zero()),
                            ShapeHandle2::new(shapes),
                            self.polyminis_cgroup, GeometricQueryType::Proximity(0.0),
                            PolyminiPhysicsData{});
    }

    pub fn apply(&mut self, id: usize, action: &PhysicsAction)
    {
        let _ = self.c.collision_object(id);
        match action
        {
            _ =>
            {
            }
        }
    }

    pub fn step(&mut self)
    {
        self.c.update();
        //
        // Idea: We handle collisions, and undo movements and reupdate
        // so things stay in the same place but the collision is recorded
        //self.c.update();
    }

    fn get(&self, id: usize) -> Option<&CollisionObject2<f32, PolyminiPhysicsData>>
    {
        self.c.collision_object(id)
    }
}
