//
// LAYERING - THIS IS THE ONLY PLACE ALLOWED TO INCLUDE ncollide / nalgebra
extern crate nalgebra;
extern crate ncollide;

use self::nalgebra::{Isometry2,  Point2, Vector1, Vector2, zero};
use self::nalgebra::{Translation, Rotation};
use self::nalgebra::{distance};

use self::ncollide::query::{Proximity, Contact};
use self::ncollide::shape::{Compound, Compound2, Cuboid, Shape2, ShapeHandle2};
use self::ncollide::world::{CollisionWorld, CollisionWorld2,
                            CollisionGroups, CollisionObject2, GeometricQueryType};
//
//

use std::f32::consts;
use std::cell::{Cell as std_Cell, RefCell as std_RefCell};

use ::actuators::*;
use ::serialization::*;
use ::types::*;
use ::uuid::PUUID;

// Polymini Physics Object Type
#[derive(Debug)]
enum PPOType 
{
    Polymini,
    StaticObject,
}

#[derive(Clone, Copy, Debug)]
struct CollisionEvent
{
    id_1: PUUID, 
    id_2: PUUID,
    pos_1: Vector2<f32>,
    pos_2: Vector2<f32>
    //TODO: More?
}
impl Serializable for CollisionEvent
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("ID_1".to_owned(), self.id_1.to_json());
        json_obj.insert("ID_2".to_owned(), self.id_2.to_json());

        if (ctx.has_flag(PolyminiSerializationFlags::PM_SF_DEBUG))
        { 
            json_obj.insert("POS_1".to_owned(), self.pos_1.serialize(ctx));
            json_obj.insert("POS_2".to_owned(), self.pos_2.serialize(ctx));
        }
        Json::Object(json_obj)
    }
}
impl Serializable for Vector2<f32>
{
    fn serialize(&self, _: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("x".to_owned(), self.x.to_json());
        json_obj.insert("y".to_owned(), self.y.to_json());
        Json::Object(json_obj)
    }
}

#[derive(Debug, Clone, Copy)]
struct StaticCollider
{
    uuid: PUUID,
    position: (f32, f32),
    dimensions: (u8, u8),
}
impl Serializable for StaticCollider
{
    fn serialize(&self, _: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("ID".to_owned(), self.uuid.to_json());
        json_obj.insert("Position".to_owned(), self.position.to_json());
        json_obj.insert("Dimensions".to_owned(), self.dimensions.to_json());
        Json::Object(json_obj)
    }
}



#[derive(Debug)]
struct PolyminiPhysicsData
{
    ppo_type: PPOType,
    initial_pos: std_Cell<Isometry2<f32>>,
    dimensions: std_Cell<Vector2<f32>>,
    collision_events: std_RefCell<Vec<CollisionEvent>>,
    looped: std_Cell<bool>,
}
impl PolyminiPhysicsData 
{
    fn new_for_polymini(pos: Vector2<f32>, dimensions: Vector2<f32>) -> PolyminiPhysicsData
    {
        PolyminiPhysicsData
        {
            ppo_type: PPOType::Polymini,
            initial_pos: std_Cell::new(Isometry2::new(pos, Vector1::new(0.0))),
            dimensions: std_Cell::new(dimensions),
            collision_events: std_RefCell::new(vec![]),
            looped: std_Cell::new(false),
        }
    }
    fn new_static_object(pos: Vector2<f32>, dimensions: Vector2<f32>) -> PolyminiPhysicsData
    {
        PolyminiPhysicsData
        {
            ppo_type: PPOType::StaticObject,
            initial_pos: std_Cell::new(Isometry2::new(pos, Vector1::new(0.0))),
            dimensions: std_Cell::new(dimensions),
            collision_events: std_RefCell::new(vec![]),
            looped: std_Cell::new(false),
        }
    }
}

pub struct PhysicsActionAccumulator
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

    fn accumulate(&mut self, dir: Direction, impulse: f32, torque: f32)
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
                //self.spin += impulse;
                // Error?
            }
            _ => panic!("Incorrect direction for impulse {:?}", dir)
        }

        self.spin += torque;
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

        
        if max == vertical_impulse
        {
            dir = Direction::VERTICAL;
            v = self.vertical_impulse; 
        }
        else if max == horizontal_impulse
        {
            dir = Direction::HORIZONTAL;
            v = self.horizontal_impulse; 
        }
        else if max == spin
        {
            dir = Direction::ROTATION;
            v = self.spin; 
        }

        if v != 0.0
        {
            Action::MoveAction(MoveAction::Move(dir, v, 0.0))
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

fn serialize_vector(v: Vector2<f32>) -> Json
{
    (v.x, v.y).to_json()
}


// Physics
//
pub struct Physics
{
    uuid: PUUID,

    ncoll_dimensions: Vector2<f32> ,
    ncoll_pos: Vector2<f32>,
    ncoll_starting_pos: Vector2<f32>,

    orientation: u8,
    collisions: Vec<CollisionEvent>,

    move_succeded: bool,
    last_action: Action,
}
impl Physics
{
    // Private
    fn build_bounding_box(&self) -> Compound2<f32>
    {
        // First pass, shape = box of dimensions = Physics.dimensions

        // Shapes are anchored in the center (unlike Physics which is top-left anchored)
        // so we need to correct for that
        let c_dimensions = (self.ncoll_dimensions.x as f32 / 2.0, self.ncoll_dimensions.y as f32 / 2.0);


        let rect = ShapeHandle2::new(Cuboid::new(Vector2::new(c_dimensions.0 as f32,
                                                              c_dimensions.1 as f32)));
        let iso = Isometry2::new(zero(), zero());

        //TODO: Create several shapes to match the morphology closely
        Compound::new(vec![(iso, rect)])
    }

    // Public
    pub fn new(uuid: PUUID, dimensions: (u8, u8), x: f32, y: f32, orientation: u8) -> Physics
    {
        let nc_dims = dimensions_sim_to_ncoll(dimensions);
        let nc_pos = Vector2::new(x + nc_dims.x / 2.0, y + nc_dims.y / 2.0);

        Physics
        {
            uuid: uuid,

            ncoll_dimensions: nc_dims,
            ncoll_pos: nc_pos,
            ncoll_starting_pos: nc_pos,

            orientation: orientation,
            collisions: vec![],


            move_succeded: true,
            last_action: Action::NoAction,
        }
    }

    pub fn reset(&mut self, pos: (f32, f32))
    {
        let n_pos = Vector2::new( pos.0 + self.ncoll_dimensions.x / 2.0,
                                  pos.1 + self.ncoll_dimensions.y / 2.0 );
        info!("Reseting Physics - New Pos: {} (Old Pos: {}", n_pos, self.ncoll_pos);

        self.ncoll_pos = n_pos;
        self.ncoll_starting_pos = n_pos;
    }

    pub fn get_starting_pos(&self) -> (f32, f32)
    {
        (self.ncoll_starting_pos.x - self.ncoll_dimensions.x / 2.0,
         self.ncoll_starting_pos.y - self.ncoll_dimensions.y / 2.0)
    }
    pub fn get_pos(&self) -> (f32, f32)
    {
        (self.ncoll_pos.x - self.ncoll_dimensions.x / 2.0,
         self.ncoll_pos.y - self.ncoll_dimensions.y / 2.0)
    }

    pub fn get_distance_moved(&self) -> f32
    {
        nalgebra::distance(self.ncoll_pos.as_point(), self.ncoll_starting_pos.as_point())
    }

    pub fn get_orientation(&self) -> Direction 
    {
        let directions = [Direction::UP, Direction::RIGHT, Direction::DOWN, Direction::LEFT];
        directions[self.orientation as usize]
    }

    pub fn get_move_succeded(&self) -> bool
    {
        self.move_succeded
    }

    // Attempt to add rotation / translation to our physics object
    pub fn act_on(&mut self, actions: &ActionList, physics_world: &mut PhysicsWorld)
    {
        // Only move actions are relevant to us
        let accum = actions.iter().fold(PhysicsActionAccumulator::new(),
                                       |mut accum, action|
                                       {
                                           match action
                                           {
                                               &Action::MoveAction(MoveAction::Move(d, i, t)) =>
                                               {
                                                   accum.accumulate(d, i, t);
                                               },
                                               _ =>
                                               {
                                                   //Ignore
                                               }
                                           }
                                           accum
                                       });

        self.last_action = accum.to_action();
        physics_world.apply(self.uuid, self.last_action);
    }

    // Update our information from the result of the simulation
    pub fn update_state(&mut self, physics_world: &PhysicsWorld)
    {
        let o = physics_world.get(self.uuid).unwrap();

        // Update position
        self.ncoll_pos = o.position.translation;

        // If an attempt to move was made, but we didn't move, update
        // last move succeded
        //
        self.move_succeded =  (self.collisions.len() == 0);
        

        // Copy collision events over and nuke the list
        self.collisions.clear();

        for ev in o.data.collision_events.borrow().iter()
        {
            self.collisions.push(*ev);
        }

        // Set our new initial position
        o.data.initial_pos.set(o.position);

        // Nuke'm
        o.data.collision_events.borrow_mut().clear();

        //TODO: Calculate orientation,
    }
}
impl Serializable for Physics
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        json_obj.insert("ID".to_owned(), self.uuid.to_json());

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("Dimensions".to_owned(), serialize_vector(self.ncoll_dimensions));
            json_obj.insert("StartingPos".to_owned(), self.get_starting_pos().to_json());
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
            json_obj.insert("Position".to_owned(), self.get_pos().to_json());
            let mut ev_arr = pmJsonArray::new();
            for ev in &self.collisions
            {
                ev_arr.push(ev.serialize(ctx));
            }
            json_obj.insert("collisions".to_owned(), Json::Array(ev_arr));

            json_obj.insert("last_action".to_owned(), self.last_action.to_json());
        }
        Json::Object(json_obj)
    }
}

// Physics World
pub struct PhysicsWorld
{
    world: CollisionWorld2<f32, PolyminiPhysicsData>,
    static_objects: Vec<StaticCollider>,
    dimensions: (f32, f32),

    //
    polyminis_cgroup: CollisionGroups,
    objects_cgroup: CollisionGroups
}
impl PhysicsWorld
{
    pub fn new() -> PhysicsWorld
    {
        PhysicsWorld::new_with_dimensions((100.0, 100.0))
    }

    pub fn new_with_dimensions(dimensions: (f32, f32)) -> PhysicsWorld
    {
        let mut col_w = CollisionWorld::new(0.02, false);

        let mut pcg = CollisionGroups::new();
        pcg.set_membership(&[1]);

        let mut ocg = CollisionGroups::new();
        ocg.set_membership(&[2]);
        ocg.set_whitelist(&[1]);

        let ph_w = PhysicsWorld { world: col_w,
                                  static_objects: vec![],
                                  polyminis_cgroup: pcg,
                                  objects_cgroup: ocg,
                                  dimensions: dimensions,
        };
        ph_w
    }

    pub fn add_object(&mut self, uuid: PUUID, position: (f32, f32),  dimensions: (u8, u8))
    {
        let nc_dim = dimensions_sim_to_ncoll(dimensions);
        let rect = Cuboid::new( Vector2::new(nc_dim.x / 2.0, nc_dim.y / 2.0) );
        let nc_pos = Vector2::new(position.0 + nc_dim.x / 2.0, position.1 + nc_dim.y / 2.0);

        self.world.deferred_add(uuid,
                            Isometry2::new(nc_pos, zero()),
                            ShapeHandle2::new(rect),
                            self.objects_cgroup, GeometricQueryType::Contacts(0.0),
                            PolyminiPhysicsData::new_static_object(nc_pos, nc_dim));

        self.static_objects.push(StaticCollider { uuid: uuid, position: position, dimensions: dimensions });
    }

    pub fn add(&mut self, physics: &Physics) -> bool
    {
        let shapes = physics.build_bounding_box();

        self.world.deferred_add(physics.uuid,
                            Isometry2::new(physics.ncoll_pos, zero()),
                            ShapeHandle2::new(shapes),
                            self.polyminis_cgroup, GeometricQueryType::Contacts(0.0),
                            PolyminiPhysicsData::new_for_polymini(physics.ncoll_pos, physics.ncoll_dimensions));
        let v = !self.finish_adding();
        if v 
        {
            error!("Removing {}", physics.uuid);
            self.remove(physics);
            self.finish_adding();
            warn!("Removed");
            false
        }
        else
        {
            true
        }
    }

    pub fn remove(&mut self, physics: &Physics)
    {
        self.world.deferred_remove(physics.uuid);
        self.world.update();
    }

    pub fn apply(&mut self, id: usize, action: Action)
    {
        let mut new_pos;
        {
            let p_obj = self.world.collision_object(id).unwrap();
            match action
            {
                Action::MoveAction(MoveAction::Move(Direction::ROTATION, spin, _)) =>
                {
                    let mut m = 1.0;
                    if spin < 0.0
                    {
                        m = -1.0;
                    }
                    debug!("Before rotation {}", p_obj.position.translation);
                    new_pos = p_obj.position;
                    new_pos.set_rotation((Vector1::new(consts::FRAC_PI_2) * m));
                    debug!("Before rotation {}", new_pos.translation);
                },
                Action::MoveAction(MoveAction::Move(Direction::VERTICAL, impulse, _)) =>
                {
                    let mut m = 1.0;
                    if impulse < 0.0
                    {
                        m = -1.0;
                    }
                    new_pos = p_obj.position.append_translation(&Vector2::new(0.0, m*1.0));
                },
                Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, impulse, _)) =>
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

    pub fn step(&mut self) -> bool
    {
        self.step_internal(true, false)
    }

    pub fn finish_adding(&mut self) -> bool
    {
        self.step_internal(false, true)
    }

    // NOTE:
    // Placement means we retry positioning objects that are colliding 
    fn step_internal(&mut self, record_events_param: bool, placement: bool) -> bool
    {
        debug!("Physics Step Internal");
        let mut record_events = record_events_param;

        // Idea: We handle collisions, and undo movements and reupdate
        // so things stay in the same place but the collision is recorded
        //
        let mut loops = 0;
        let max_loops = if placement { 500 } else { 200 };

        let mut phys_capture: Vec<Json>;
        #[cfg(physics_capture)]
        {
            phys_capture = vec![];
        }

        loop
        {
            self.world.update();
            let mut collisions = false;
            let mut corrections = vec![];
            for (pair_inx, coll_data) in self.world.contacts().enumerate()
            {
                let (object_1, object_2, contact_data) = coll_data;

                let mut can_skip = true;
                if contact_data.depth > 0.005 
                {
                    can_skip = false;
                }

                if loops >= (max_loops - 5)
                {
                    error!("Dumping collisions Loop({}) {} {} {}", loops, object_1.uid, object_2.uid, contact_data.depth);
                    error!("Dumping collisions Loop {} {}", contact_data.world1, contact_data.world2);
                }
                if can_skip
                {
                    continue;
                }

                let mut n_pos = object_1.data.initial_pos.get();
                let mut n_pos_2 = object_2.data.initial_pos.get();

                if max_loops - loops < 3
                {
                    error!("Start dumping: {} {}", object_1.position, object_2.position);
                }


                if (placement)
                {
                    let mut m = ( loops as f32 / 4.0 ).ceil();
                    if m > 10.0
                    {
                        m = 10.0;
                    }
                    let displacements = vec![Vector2::new( m,     0.0),
                                             Vector2::new( 0.0,  -1.0*m),
                                             Vector2::new(-1.0*m, 0.0),
                                             Vector2::new( 0.0,   m)];

                    let mut target_obj;
                    let mut other_obj;
                    let mut target_obj_new_pos;

                    match object_1.data.ppo_type
                    {
                        PPOType::Polymini =>
                        {
                            match object_2.data.ppo_type
                            {
                                PPOType::Polymini =>
                                {
                                    // If both objects are Polyminis, and we're placing,
                                    // we move the one with the highest ID, to keep it as
                                    // deterministic as possible
                                    if (object_1.uid > object_2.uid)
                                    {
                                        target_obj = object_1;
                                        other_obj = object_2;
                                    }
                                    else
                                    {
                                        target_obj = object_2;
                                        other_obj = object_1;
                                    }
                                }
                                PPOType::StaticObject =>
                                {
                                    target_obj = object_1;
                                    other_obj = object_2;
                                }
                            }
                        }
                        PPOType::StaticObject =>
                        {
                            target_obj = object_2;
                            other_obj = object_1;
                        }
                    }
                    target_obj_new_pos = target_obj.data.initial_pos.get();
                    target_obj_new_pos.translation +=  displacements[ (loops + pair_inx) % displacements.len() ];
                    debug!("New Position: {}",
                           target_obj_new_pos.translation.serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));

                    let left   = target_obj_new_pos.translation.x - target_obj.data.dimensions.get().x / 2.0;
                    let right  = target_obj_new_pos.translation.x + target_obj.data.dimensions.get().x / 2.0;
                    let bottom = target_obj_new_pos.translation.y - target_obj.data.dimensions.get().y / 2.0;
                    let top    = target_obj_new_pos.translation.y + target_obj.data.dimensions.get().y / 2.0;

                    if left < 0.0
                    {
                        target_obj_new_pos.translation.x = target_obj.data.dimensions.get().x / 2.0; 
                    }

                    if right > self.dimensions.0 
                    {
                        target_obj_new_pos.translation.x = self.dimensions.0 - target_obj.data.dimensions.get().x / 2.0; 
                    }

                    if bottom < 0.0
                    {
                        target_obj_new_pos.translation.y = target_obj.data.dimensions.get().y / 2.0; 
                    }

                    if  top > 100.0 
                    {
                        target_obj_new_pos.translation.y = self.dimensions.1 - target_obj.data.dimensions.get().y / 2.0; 
                    }

                    target_obj.data.initial_pos.set(target_obj_new_pos);
                    corrections.push((target_obj.uid, target_obj_new_pos, target_obj.data.dimensions.get(), other_obj.uid));
                }
                else
                {
                    corrections.push((object_1.uid, n_pos, object_1.data.dimensions.get(), object_2.uid));
                    corrections.push((object_2.uid, n_pos_2, object_2.data.dimensions.get(), object_1.uid));
                }

                let ev = CollisionEvent
                {
                    id_1: object_1.uid,
                    id_2: object_2.uid,
                    pos_1: object_1.position.translation,
                    pos_2: object_2.position.translation
                };

                debug!("{}", 
                       ev.serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));
                debug!("Object Dimensions {} - {}", object_1.data.dimensions.get(), object_2.data.dimensions.get());

                if record_events
                {

                    object_1.data.collision_events.borrow_mut().push(ev);
                    object_2.data.collision_events.borrow_mut().push(ev);
                }

                collisions = true;
            }

            // Only record collision events on the first pass, not on the rewind passes
            record_events = false;
            if !collisions
            {
                break;
            }

            loops += 1;
            if loops >= (max_loops - 5)
            {

                error!("Last set of Corrections: ");
                for c in &corrections
                {
                    error!("{} {:?} {} {}", c.0, c.1, c.2, c.3);
                }

                if loops == max_loops
                {
                    if placement
                    {
                        // This object can't be placed correctly
                        return false;
                    }

                    panic!("Probably caught in endless loop");
                }
            }

            for c in &corrections
            {
                self.world.deferred_set_position(c.0, c.1);
            }

            debug!("Looping");
        }
        return true;
    }

    fn get(&self, id: usize) -> Option<&CollisionObject2<f32, PolyminiPhysicsData>>
    {
        self.world.collision_object(id)
    }
}
impl Serializable for PhysicsWorld
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        
        let mut json_obj = pmJsonObject::new();
        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            // Serialize Static objects
            let mut so_arr = pmJsonArray::new();
            for static_obj in &self.static_objects
            {
                so_arr.push(static_obj.serialize(ctx));
            }
            json_obj.insert("StaticObjects".to_string(), Json::Array(so_arr));
        }
        Json::Object(json_obj)
    }
}


#[cfg(test)]
mod test
{
    extern crate env_logger;
    use super::*;
    use ::actuators::*;
    use ::genetics::*;
    use ::morphology::*;
    use ::types::*;


    #[test]
    fn test_placement()
    {
        let mut physical_world = PhysicsWorld::new();
        let mut physics = Physics::new(1, (4, 4), 0.0, 0.0, 0); 
        physical_world.add_object(2, (0.0, 0.0), (2, 2));
        physical_world.add(&physics);
        physics.update_state(&physical_world);

        assert_eq!(physics.get_pos(), (0.0, 2.0));
    }

    #[test]
    fn test_rotation_and_translation()
    {
        let _ = env_logger::init();
        let mut physical_world = PhysicsWorld::new();
        let mut physics = Physics::new(1, (4, 4), 0.0, 0.0, 0); 
        physical_world.add(&physics);
        physics.update_state(&physical_world);

        physics.act_on(&vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2, 2.0)),
                             Action::MoveAction(MoveAction::Move(Direction::VERTICAL, 1.1, 0.0))],
                       &mut physical_world);
        physical_world.step();
        physics.update_state(&physical_world);
        assert_eq!(physics.get_pos(), (0.0, 0.0));
        physics.act_on(&vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2, 0.0)),
                             Action::MoveAction(MoveAction::Move(Direction::VERTICAL, 1.1, 0.0))],
                       &mut physical_world);

        physical_world.step();
        physics.update_state(&physical_world);
        assert_eq!(physics.get_pos(), (1.0, 0.0));
        physics.act_on(&vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2, 0.0)),
                             Action::MoveAction(MoveAction::Move(Direction::VERTICAL, 1.1, 2.0))],
                       &mut physical_world);
        assert_eq!(physics.get_pos(), (1.0, 0.0));
    }

    fn test_physics_undoing()
    {}

    fn test_movement_accumulator_master(actions: ActionList, expected_impulse: f32, expected_direction: Direction)
    {
        //
        let accum = actions.iter().fold(PhysicsActionAccumulator::new(),
                                        |mut accum, action|
                                        {
                                            match action
                                            {
                                                &Action::MoveAction(MoveAction::Move(d, i, t)) =>
                                                {
                                                    accum.accumulate(d, i, t);
                                                },
                                                _ =>
                                                {
                                                    //Ignore
                                                }
                                            }
                                            accum
                                       });
        match accum.to_action()
        {
            Action::MoveAction(MoveAction::Move(dir, impulse, _)) =>
            {
                assert_eq!(dir, expected_direction);
                assert!( (impulse - expected_impulse) < 0.001);
            },
            WrongAction =>
            {
                panic!("Result of PhysicAccumulatorIncorrect - {:?}", WrongAction);
            }
        }
    }

    #[test]
    fn test_movement_accumulator_1()
    {
        test_movement_accumulator_master( vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2, 0.0)),
                                               Action::MoveAction(MoveAction::Move(Direction::VERTICAL, 1.1, 0.0))],
                                          1.2,
                                          Direction::HORIZONTAL);
    }

    #[test]
    fn test_movement_accumulator_2()
    {
        test_movement_accumulator_master( vec![Action::MoveAction(MoveAction::Move(Direction::HORIZONTAL, 1.2, 2.0)),
                                               Action::MoveAction(MoveAction::Move(Direction::VERTICAL, 1.1, 0.0))],
                                          2.0,
                                          Direction::ROTATION);
    }


    #[test]
    fn test_accum_from_actuators_1()
    {
        let ac_list = vec![ Actuator::new(ActuatorTag::MoveHorizontal, 0, (0, 1)), 
                            Actuator::new(ActuatorTag::MoveHorizontal, 1, (1, 1)) ];
        let mut actions = vec![];

        for actuator in ac_list
        {
            actions.push(actuator.get_action(1.1));
        }

        test_movement_accumulator_master(actions, 2.2, Direction::HORIZONTAL);
    }

    #[test]
    fn test_accum_from_actuators_2()
    {
        let ac_list = vec![ Actuator::new(ActuatorTag::MoveHorizontal, 0, (0, 2)), 
                            Actuator::new(ActuatorTag::MoveHorizontal, 1, (1, 1)) ];
        let mut actions = vec![];

        for actuator in ac_list
        {
            actions.push(actuator.get_action(1.1));
        }

        test_movement_accumulator_master(actions, 3.3, Direction::ROTATION);
    }
}
