use ::actuators::*;
use ::physics::Vector2;
use ::serialization::*;
use ::uuid::PUUID;

use std::collections::HashMap;
use std::cmp;
use std::fmt;
use std::f32;

// Temperature Tracker for Polyminis 


struct ThermoActionAccum
{
    delta: f32,
}
impl ThermoActionAccum
{
    fn new() -> ThermoActionAccum
    {
        ThermoActionAccum { delta: 0.0 }
    }
    fn to_action(&self) -> Action
    {
        Action::ThermalAction(ThermalAction::Change(self.delta * 0.2))
    }
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

        // Find actions related to thermal
        let accum = actions.iter().fold(ThermoActionAccum::new(),
                                       |mut accum, action|
                                       {
                                           match action
                                           {
                                               &Action::ThermalAction(ThermalAction::Change(d)) =>
                                               {
                                                   accum.delta += d;
                                               },
                                               _ => {}
                                           }
                                           accum
                                       });

        match thermo_world.thermo_objects.get_mut(&self.uuid)
        {
            Some(ref mut therm_object) =>
            {
                therm_object.position = position;
            },
            None =>
            {
                error!("Thermo - FATAL - UUID not found! - {:?}", self.uuid);
            }
        }

        thermo_world.apply(self.uuid, accum.to_action());
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
                error!("Thermo - FATAL - UUID not found! - {:?}", self.uuid);
            }
        }
    }

    pub fn inside_range(&self) -> bool
    {
        self.min <= self.current && self.current <= self.max
    }

    pub fn delta_from_range(&self) -> f32
    {
        if self.current < self.min
        {
            self.min - self.current
        }
        else if self.current > self.max
        {
            self.current - self.max
        }
        else
        {
            0.0
        }
    }

    pub fn current_temperature(&self) -> f32
    {
        self.current
    }
}
impl Serializable for Thermo
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();

        if ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC) ||
           ctx.has_flag(PolyminiSerializationFlags::PM_SF_DEBUG)
        {
            json_obj.insert("Current".to_owned(), Json::F64(self.current as f64));
            json_obj.insert("InRange".to_owned(), Json::Boolean(self.inside_range()));
        }

        if !ctx.has_flag(PolyminiSerializationFlags::PM_SF_DYNAMIC) ||
            ctx.has_flag(PolyminiSerializationFlags::PM_SF_DEBUG)
        {
            json_obj.insert("Min".to_owned(), Json::F64(self.min as f64));
            json_obj.insert("Max".to_owned(), Json::F64(self.max as f64));
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
        let (gc_x, gc_y) = ThermoWorld::coord_to_grid_position(pos, self.dimensions, self.thermo_grid.len(), self.thermo_grid[0].len());
        let grid_temp = self.thermo_grid[gc_x][gc_y];
        let d = grid_temp - thermo.current;
        thermo.current += 0.25*d;
        let mut data = ThermoData { uuid: thermo.uuid, position: pos, emmit_intensity: 5.0, 
                                    current_temperature: thermo.current, is_individual: true };
        self.thermo_objects.insert(thermo.uuid, data);
        true
    }

    pub fn add_object(&mut self, uuid: PUUID, position: (f32, f32), current_temperature: f32, intensity: f32)
    {
        let obj = ThermoData { uuid: uuid, position: position, emmit_intensity: intensity, current_temperature: current_temperature, is_individual: false };
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
                let gcoords = ThermoWorld::coord_to_grid_position(obj.position, self.dimensions, self.thermo_grid.len(), self.thermo_grid[0].len());

                let cx = cmp::min(cmp::max(gcoords.0, 0), x_len -1);
                let cy = cmp::min(cmp::max(gcoords.1, 0), y_len -1);


                let mut grid_v = self.thermo_grid[cx][cy];

                let d = grid_v - obj.current_temperature;
                debug!("Thermo Step: grid_v = {}", grid_v);
                obj.current_temperature += 0.25*d;
                // Clamp
                obj.current_temperature.max(0.0).min(1.0);
                /*
                let mut grid_v_delta = grid_v - self.thermo_grid[gcoords.0][gcoords.1];
                grid_v_delta /= 100.0;
                if grid_v_delta >= 0.001
                {
                    self.thermo_grid[gcoords.0][gcoords.1] += grid_v_delta  ; // Polyminis affect temperature just a little bit
                    self.thermo_grid[gcoords.0][gcoords.1].min(1.0).max(0.0);
                }
                */
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
            if obj.is_individual
            {
                // Skip individuals
                continue;
            }

            let gcoords = ThermoWorld::coord_to_grid_position(obj.position, self.dimensions, self.thermo_grid.len(), self.thermo_grid[0].len());
            for t_x in 0..self.thermo_grid.len()
            {
                for t_y in 0..self.thermo_grid[t_x].len()
                {
                    let diff_x = (t_x as f32 - gcoords.0 as f32).abs();
                    let diff_y = (t_y as f32 - gcoords.1 as f32).abs();

                    if diff_x <= 0.001 && diff_y <= 0.001
                    {
                        self.thermo_grid[t_x][t_y] = obj.current_temperature;
                    }
                    else
                    {
                        let mut grid_v = self.thermo_grid[t_x][t_y];
                        let d_temp = (obj.current_temperature - grid_v);

                        let v = (d_temp / diff_x.max(diff_y));
                        grid_v += if diff_x < obj.emmit_intensity && diff_y < obj.emmit_intensity { v } else { 0.0 }; 
                        
                        self.thermo_grid[t_x][t_y] = grid_v.min(1.0).max(0.0);
                    }
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
                match action
                {
                    Action::ThermalAction(ThermalAction::Change(d)) =>
                    {
                        therm_obj.current_temperature += d;
                    },
                    _ =>
                    {
                        error!("An action of type {:?} ended up in Thermal World apply", action);
                    }
                }
            }
            None =>
            {
                panic!("ThermoWorld - FATAL - UUID not found! - {:?}", uuid);
            }
        }
    }

    fn coord_to_grid_position(position: (f32, f32), dims: (f32, f32), x_len: usize, y_len: usize) -> (usize, usize)
    {
        if (position.0 < 0.0 || position.1 < 0.0)
        {
            debug!("{:?}", position);
        }

        let mut x =
            if position.0 >= 0.0
            {
                (position.0 / dims.0 * x_len as f32).floor() as usize
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
                (position.1 / dims.1 * y_len as f32).floor() as usize
            }
            else
            {
                0
            };

        if (y == y_len)
        {
            y -= 1;
        }

        debug!("Adding Thermo Object {:?} to {:?}", position, (x,y));
        (x, y)
    }
}
impl fmt::Debug for ThermoWorld
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        for t_x in 0..self.thermo_grid.len()
        {
            write!(f, "\n");
            for t_y in 0..self.thermo_grid[t_x].len()
            {
                write!(f, "{:.*} ", 2, self.thermo_grid[t_x][t_y]);
            }
        }
        write!(f, "\n")
    }
}
impl fmt::Display for ThermoWorld
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}
impl Serializable for ThermoWorld
{
    fn serialize(&self, ctx: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        let mut grid_arr = pmJsonArray::new();
        for t_x in 0..self.thermo_grid.len()
        {
            for t_y in 0..self.thermo_grid[t_x].len()
            {
                grid_arr.push(Json::F64(self.thermo_grid[t_x][t_y] as f64));
            }
        }
        json_obj.insert("Dimensions".to_owned(), Vector2::new(self.thermo_grid.len() as f32, self.thermo_grid[0].len() as f32).serialize(ctx));
        json_obj.insert("Grid".to_owned(), Json::Array(grid_arr));
        Json::Object(json_obj)
    }
}


#[cfg(test)]
mod test
{
    extern crate env_logger;

    use super::*;

    #[test]
    fn test_thermal_update()
    {
        let _ = env_logger::init();
        let mut th_world = ThermoWorld::new_with_dimensions((100.0, 100.0), 0.3141);
        debug!("{}", th_world);
        debug!("\n\n{}", th_world);
        th_world.add_object(1, (0.0, 0.0), 0.99, 5.0);
        debug!("\n\n{}", th_world);
        th_world.add_object(2, (99.0, 99.0), 0.20, 5.0);
        debug!("\n\n{}", th_world);
    }

 //   #[test]
    fn test_thermal_update_with_object()
    {
        let _ = env_logger::init();
        let mut th_world = ThermoWorld::new_with_dimensions((100.0, 100.0), 0.3141);

        // Add objects
        th_world.add_object(1, (0.0, 0.0), 0.99, 5.0);
        th_world.add_object(2, (99.0, 99.0), 0.20, 5.0);
        debug!("{}", th_world);

        // Add Thermo component 
        let mut th_obj = Thermo::new(3141, 0.3, 0.7);
        th_world.add(&mut th_obj, (1.0, 1.0));
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));


        // Step & Update
        th_world.step();
        th_obj.update_state(&th_world);
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));

        th_world.step();
        th_obj.update_state(&th_world);
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));
    }

    //#[test]
    fn test_thermal_actions()
    {
        let _ = env_logger::init();
        let mut th_world = ThermoWorld::new_with_dimensions((100.0, 100.0), 0.3141);

        // Add objects
        th_world.add_object(1, (0.0, 0.0), 0.99, 5.0);
        th_world.add_object(2, (90.0, 90.0), 0.20, 5.0);
        debug!("{}", th_world);

        // Add Thermo component 
        let mut th_obj = Thermo::new(3141, 0.3, 0.7);
        th_world.add(&mut th_obj, (1.0, 1.0));
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));

        // Step & Update
        th_world.step();
        th_obj.update_state(&th_world);
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));

        th_obj.act_on((1.0, 1.0), &vec![Action::ThermalAction(ThermalAction::Change(-0.5))], &mut th_world);
        th_world.step();
        th_obj.update_state(&th_world);
        debug!("\n\n{}", th_obj.serialize(&mut SerializationCtx::debug()));
    }

}
