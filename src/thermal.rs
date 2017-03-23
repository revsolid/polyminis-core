use ::actuators::*;
use ::serialization::*;
use ::uuid::PUUID;

use std::collections::HashMap;
use std::f32;

// Temperature Tracker for Polyminis 


struct ThermoActionAccum
{
}

pub struct Thermo 
{
    uuid: PUUID,
    max: f32,
    current: f32,
    min: f32,
}
impl Thermo
{
    pub fn new(uuid: PUUID, min: f32, max: f32) -> Thermo
    {
        Thermo { uuid: uuid, min: min, max: max, current: (min + max)/ 2.0 }
    }

    pub fn act_on(&self, position: (f32, f32), actions: &ActionList, thermo_world: &mut ThermoWorld)
    {
        thermo_world.apply(self.uuid, Action::NoAction);
    }

    pub fn update_state(&mut self, world: &ThermoWorld)
    {
        match world.thermo_objects.get(&self.uuid)
        {
            Some(ref therm_object) =>
            {
                self.current = therm_object.current_temperature;
            }
            None =>
            {
                panic!("Thermo - FATAL - UUID not found! - {:?}", self.uuid);
            }
        }
    }

    pub fn inside_range(&self) -> bool
    {
        self.min <= self.current && self.current <= self.max
    }
}
impl Serializable for Thermo
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_STATIC)
        {
            json_obj.insert("Min".to_owned(), Json::F64(self.min as f64));
            json_obj.insert("Max".to_owned(), Json::F64(self.max as f64));
        }

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC)
        {
            json_obj.insert("Current".to_owned(), Json::F64(self.current as f64));
        }
        Json::Object(json_obj)
    }
}

// Follow the same pattern as the PhysicsWorld which is the most complex
struct ThermoData
{
    uuid: PUUID,
    current_temperature: f32,
    emmit_intensity: f32,
    position: (f32, f32),
    is_individual: bool
}
pub struct ThermoWorld 
{
    // Hash - uuid | Thermo Data
    thermo_objects: HashMap<PUUID, ThermoData>,
    
    // NxN grid of temperature 'areas'
    thermo_grid: Vec<Vec<f32>>,
    grid_square_size: f32,
    
    // World dimensions
    dimensions: (f32, f32),

    base_temperature: f32
}
impl ThermoWorld
{
    pub fn new() -> ThermoWorld
    {
        ThermoWorld::new_with_dimensions((100.0, 100.0), 0.0)
    }

    pub fn new_with_dimensions(dims: (f32, f32), base_temp: f32) -> ThermoWorld
    {
        let mut grid = vec![];
        let grid_sq_size = 10.0;
        let grid_sqs_x = (dims.0 / grid_sq_size) as usize;
        let grid_sqs_y = (dims.1 / grid_sq_size) as usize;

        for i in 0..grid_sqs_x
        {
            grid.push(vec![base_temp; grid_sqs_y]);
        }

        ThermoWorld
        {
            thermo_objects: HashMap::new(),
            thermo_grid: grid,
            grid_square_size: grid_sq_size,
            dimensions: dims,
            base_temperature: base_temp
        }
    }

    pub fn add(&mut self, thermo: &mut Thermo, pos: (f32, f32)) -> bool
    {
        let (gc_x, gc_y) = ThermoWorld::coordToGridPosition(pos, self.dimensions, self.thermo_grid.len(), self.thermo_grid[0].len());
        let grid_temp = self.thermo_grid[gc_x][gc_y];
        let mut data = ThermoData { uuid: thermo.uuid, position: pos, emmit_intensity: 0.1, 
                                    current_temperature: grid_temp, is_individual: true };
        thermo.current = grid_temp;
        self.thermo_objects.insert(thermo.uuid, data);
        true
    }

    pub fn add_object(&mut self, uuid: PUUID, position: (f32, f32), intensity: f32)
    {
        let obj = ThermoData { uuid: uuid, position: position, emmit_intensity: intensity, current_temperature: 0.0, is_individual: false };
        self.thermo_objects.insert(uuid, obj);
        self.recalculate();
    }

    pub fn step(&mut self) 
    {
        let dims = self.dimensions;
        let x_len = self.thermo_grid.len();
        let y_len = self.thermo_grid[0].len();
        for ref mut obj in self.thermo_objects.values_mut()
        {
            if obj.is_individual         
            {
                let gcoords = ThermoWorld::coordToGridPosition(obj.position, dims, x_len, y_len);
                let mut grid_v = self.thermo_grid[gcoords.0][gcoords.1];

                // N-MidPoint Algorithm (aka I'm pretty sure this has a name already)
                for i in 0..2
                {
                   grid_v = (obj.current_temperature + grid_v) / 2.0;
                }

                obj.current_temperature = grid_v;
            }
        }
    }

    pub fn recalculate(&mut self)
    {
        let mut grid = vec![];
        let grid_sq_size = 10.0;
        let grid_sqs_x = (self.dimensions.0 / grid_sq_size) as usize;
        let grid_sqs_y = (self.dimensions.1 / grid_sq_size) as usize;

        for i in 0..grid_sqs_x
        {
            grid.push(vec![self.base_temperature; grid_sqs_y]);
        }
        self.thermo_grid = grid;

        for (id, obj) in &self.thermo_objects 
        {
            for t_x in 0..self.thermo_grid.len()
            {
                for t_y in 0..self.thermo_grid[t_x].len()
                {
                    let diff_x = (t_x as f32 - obj.position.0).abs();
                    let diff_y = (t_y as f32 - obj.position.1).abs();

                    self.thermo_grid[t_x][t_y] +=  ( (obj.current_temperature) * obj.emmit_intensity / (diff_x.max(diff_y).max(1.0)));
                }
            }
        }
    }

    pub fn apply(&mut self, uuid: PUUID, action: Action)
    {
        match self.thermo_objects.get_mut(&uuid)
        {
            Some(ref mut therm_obj) =>
            {
                therm_obj.current_temperature = 0.5;
            }
            None =>
            {
                panic!("ThermoWorld - FATAL - UUID not found! - {:?}", uuid);
            }
        }
    }

    fn coordToGridPosition(position: (f32, f32), dims: (f32, f32), x_len: usize, y_len: usize) -> (usize, usize)
    {
        if (position.0 < 0.0 || position.1 < 0.0)
        {
            println!("{:?}", position);
        }

        let mut x =
            if position.0 >= 0.0
            {
                (position.0 / dims.0).floor() as usize  * x_len
            }
            else
            {
                0
            };

        if (x >= x_len)
        {
            x = x_len - 1;
        }

        let mut y =
            if position.1 >= 0.0
            {
                (position.1 / dims.1).floor() as usize  * y_len
            }
            else
            {
                0
            };

        if (y == y_len)
        {
            y -= 1;
        }

        (x, y)
    }
}
