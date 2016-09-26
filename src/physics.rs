//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE ncollide / nalgebra
extern crate nalgebra;
extern crate ncollide;

use self::nalgebra::{Isometry2,  Point2, Vector1, Vector2, zero};
use self::nalgebra::{Translation, Rotation};

use self::ncollide::narrow_phase::{ProximityHandler};
use self::ncollide::query::{Proximity};
use self::ncollide::shape::{Compound, Compound2, Cuboid, Shape2, ShapeHandle2};
use self::ncollide::world::{CollisionWorld, CollisionWorld2,
                            CollisionGroups, CollisionObject2, GeometricQueryType};
//
//

use std::f32::consts;
use std::cell::Cell as std_Cell;

use ::actuators::*;
use ::morphology::*;
use ::types::*;

// Polymini Physics Object Type
#[derive(Debug)]
enum PPOType 
{
    Polymini,
    StaticObject,
}

#[derive(Debug)]
struct PolyminiPhysicsData
{
    ppo_type: PPOType,
    initial_pos: std_Cell<Vector2<f32>>
}
impl PolyminiPhysicsData 
{
    fn new_for_polymini(pos: Vector2<f32>) -> PolyminiPhysicsData
    {
        PolyminiPhysicsData
        {
            ppo_type: PPOType::Polymini,
            initial_pos: std_Cell::new(pos),
        }
    }
    fn new_static_object(pos: Vector2<f32>) -> PolyminiPhysicsData
    {
        PolyminiPhysicsData
        {
            ppo_type: PPOType::StaticObject,
            initial_pos: std_Cell::new(pos),
        }
    }
}

struct PhysicsActionAccumulator
{
    vertical_impulse: f32,
    horizontal_impulse: f32,
    spin: f32,
}
impl PhysicsActionAccumulator
{
    fn new() -> PhysicsActionAccumulator
    {
        PhysicsActionAccumulator
        {
            vertical_impulse: 0.0,
            horizontal_impulse: 0.0,
            spin: 0.0,
        }
    }

    fn accumulate(&mut self, dir: Direction, impulse: f32)
    {
        match dir
        {
            Direction::HORIZONTAL =>
            {
                self.horizontal_impulse += impulse;
            },
            Direction::VERTICAL =>
            {
                self.vertical_impulse += impulse;
            },
            Direction::ROTATION =>
            {
                self.spin += impulse
            }
            _ => panic!("Incorrect direction for impulse {:?}", dir)
        } 
    }

    fn to_action(&self) -> Action
    {
        let vertical_impulse = self.vertical_impulse.abs();
        let horizontal_impulse = self.horizontal_impulse.abs();
        let spin = self.spin.abs();

        let mut max = vertical_impulse.max(horizontal_impulse);
        max = max.max(spin);

        let mut dir = Direction::VERTICAL;
        let mut v = 0.0;

        if max == spin
        {
            dir = Direction::ROTATION;
            v = self.spin; 
        } 
        else if max == vertical_impulse
        {
            dir = Direction::VERTICAL;
            v = self.vertical_impulse; 
        }
        else if max == horizontal_impulse
        {
            dir = Direction::HORIZONTAL;
            v = self.horizontal_impulse; 
        }

        if v > 0.0
        {
            Action::MoveAction(MoveAction::Move(dir, v))
        }
        else
        {
            Action::NoAction
        }
    }
}

// Helpers
fn dimensions_sim_to_ncoll(dim: (u8, u8)) -> Vector2<f32>
{
    Vector2::new(dim.0 as f32, dim.1 as f32)
}

fn dimensions_ncoll_to_sim(dim: Vector2<f32>) -> (u8, u8)
{
    (dim.x as u8, dim.y as u8)
}


// Physics
//
pub struct Physics
{
    uuid: usize,
    ncoll_pos: Vector2<f32>,
    orientation: u8,
    move_succeded: bool,
    ncoll_dimensions: Vector2<f32> 
}
impl Physics
{
    // Private
    fn shapes_from_morphology(m: &Morphology) -> Compound2<f32>
    {
        // First pass, shape = box of dimensions = Morph.dimensions

        // Shapes are anchored in the center (unlike Morph which is top-left anchored)
        // so we need to correct for that
        let c_dimensions = (m.get_dimensions().0 as f32 / 2.0, m.get_dimensions().1 as f32 / 2.0);
        let rect = ShapeHandle2::new(Cuboid::new(Vector2::new(c_dimensions.0 as f32,
                                                              c_dimensions.1 as f32)));
        let iso = Isometry2::new(zero(), zero());

        //TODO: Create several shapes to match the morphology closely
        Compound::new(vec![(iso, rect)])
    }

    // Public
    pub fn new(uuid: usize, dimensions: (u8, u8), x: f32, y: f32, orientation: u8) -> Physics
    {
        let nc_dims = dimensions_sim_to_ncoll(dimensions);
        let nc_pos = Vector2::new(x + nc_dims.x / 2.0, y + nc_dims.y / 2.0);

        Physics
        {
            uuid: uuid,
            ncoll_dimensions: nc_dims,
            ncoll_pos: nc_pos,
            orientation: orientation,
            move_succeded: true
        }
    }

    pub fn get_pos(&self) -> (f32, f32)
    {
        (self.ncoll_pos.x, self.ncoll_pos.y)
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
        // Only move actions are relevant to us
        let p_actions = actions.into_iter().filter(
            |x| match x
            {
                &Action::MoveAction(_) =>
                {
                        true
                },
                _ =>
                {
                        false
                }
            });

        let accum = p_actions.into_iter().fold(PhysicsActionAccumulator::new(),
                                               |mut accum, action| 
                                               {
                                                   match action
                                                   {
                                                       Action::MoveAction(MoveAction::Move(d, i)) =>
                                                       {
                                                           accum.accumulate(d, i);
                                                       },
                                                       _ =>
                                                       {
                                                           panic!("Filter must be broken");
                                                       }
                                                   }
                                                   accum
                                               });

        physics_world.apply(self.uuid, accum.to_action());
    }

    // Update our information from the result of the simulation
    pub fn update_state(&mut self, physics_world: &PhysicsWorld)
    {
        let o = physics_world.get(self.uuid).unwrap();

        self.ncoll_pos = o.position.translation;

        println!(">> {:?}", self.ncoll_pos);
        println!(">>> {:?}", o.position.rotation);
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
            println!("NO TOUCHING!!");
        }
    }
}


// Physics World
pub struct PhysicsWorld
{
    world: CollisionWorld2<f32, PolyminiPhysicsData>,

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

        let ph_w = PhysicsWorld { world: col_w,
                                  polyminis_cgroup: pcg,
                                  objects_cgroup: ocg
        };
        ph_w
    }

    pub fn add_object(&mut self, uuid: usize, position: (f32, f32),  dimensions: (u8, u8))
    {
        let nc_dim = dimensions_sim_to_ncoll(dimensions);
        let rect = Cuboid::new(nc_dim);
        let nc_pos = Vector2::new(position.0 + nc_dim.x / 2.0, position.1 + nc_dim.y / 2.0);

        self.world.deferred_add(uuid,
                            Isometry2::new(nc_pos, zero()),
                            ShapeHandle2::new(rect),
                            self.objects_cgroup, GeometricQueryType::Proximity(0.0),
                            PolyminiPhysicsData::new_static_object(nc_pos));
    }

    pub fn add(&mut self, physics: &Physics, morph: &Morphology)
    {
        let shapes = Physics::shapes_from_morphology(morph);

        //TODO: QueryType
        self.world.deferred_add(physics.uuid,
                            Isometry2::new(physics.ncoll_pos, zero()),
                            ShapeHandle2::new(shapes),
                            self.polyminis_cgroup, GeometricQueryType::Proximity(0.0),
                            PolyminiPhysicsData::new_for_polymini(physics.ncoll_pos))

    }

    pub fn apply(&mut self, id: usize, action: Action)
    {
        let new_pos;
        {
            let p_obj = self.world.collision_object(id).unwrap();
            p_obj.data.initial_pos.set(p_obj.position.translation);
            match action
            {
                Action::MoveAction(MoveAction::Move(Direction::ROTATION, spin)) =>
                {
                    let mut m = 1.0;
                    if spin < 0.0
                    {
                        m = -1.0;
                    }
                    new_pos = p_obj.position.prepend_rotation(&(Vector1::new(consts::FRAC_PI_2) * m));
                },
                Action::MoveAction(MoveAction::Move(Direction::VERTICAL, impulse)) =>
                {
                    let mut m = 1.0;
                    if impulse < 0.0
                    {
                        m = -1.0;
                    }
                    new_pos = p_obj.position.append_translation(&Vector2::new(0.0, m*1.0));
                },
                Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, impulse)) =>
                {
                    let mut m = 1.0;
                    if impulse < 0.0
                    {
                        m = -1.0;
                    }
                    new_pos = p_obj.position.append_translation(&Vector2::new(m*1.0, 0.0));
                },
                Action::NoAction =>
                {
                    new_pos = p_obj.position;
                },
                _ =>
                {
                    panic!("Incorrect Action {:?} applied to Physics World", action);
                }
            }
        }
        self.world.deferred_set_position(id, new_pos)
    }

    pub fn step(&mut self)
    {
        let mut record_events = true;

        // Idea: We handle collisions, and undo movements and reupdate
        // so things stay in the same place but the collision is recorded
        //self.world.update();
        //
        loop
        {
            self.world.update();
            let mut collisions = false;
            let mut corrections = vec![];
            for coll_data in self.world.proximity_pairs()
            {
                let (object_1, object_2, _) = coll_data;
                println!("{:?} {:?}", object_1.data.initial_pos, object_1.position.translation);
                println!("{:?} {:?}", object_2.data.initial_pos, object_2.position.translation);

                let mut n_pos = object_1.position;
                n_pos.translation = object_1.data.initial_pos.get();
                corrections.push((object_1.uid, n_pos));

                let mut n_pos_2 = object_2.position;
                n_pos_2.translation = object_2.data.initial_pos.get();
                corrections.push((object_2.uid, n_pos_2));

                if record_events
                {
                    // Record Event
                }

                collisions = true;
            }

            for c in corrections
            {
                self.world.deferred_set_position(c.0, c.1);
            }

            // Only record collision events on the first pass, not on the rewind passes
            record_events = false;
            if !collisions
            {
                break;
            }
        }
    }

    fn get(&self, id: usize) -> Option<&CollisionObject2<f32, PolyminiPhysicsData>>
    {
        self.world.collision_object(id)
    }
}
