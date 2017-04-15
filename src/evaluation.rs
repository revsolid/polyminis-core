use ::actuators::*;
use ::instincts::*;
use ::serialization::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

//TODO Maybe use naming or sub-enums?
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FitnessStatistic
{
    // Util and Control
    NoOp,

    // From Actions
    Moved,
    ConsumedFoodSource,

    PositionVisited((u32, u32)),

    // Epoch-Wide
    DistanceTravelled(u32),

    // Body
    TotalCells(usize),


    // Position
    FinalPosition(u8, u8),

    Died(u32, u32),
}
impl FitnessStatistic
{
    pub fn new_from_action(action: &Action) -> FitnessStatistic
    {
        match *action
        {
            Action::MoveAction(_) =>
            {
                return FitnessStatistic::Moved
            }
            _ => 
            {
                return FitnessStatistic::NoOp
            }
        }
    }
}
impl Serializable for FitnessStatistic
{
    fn serialize(&self, _: &mut SerializationCtx) -> Json
    {
        self.to_string().to_json()
    }
}
impl fmt::Display for FitnessStatistic 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FitnessEvaluator
{
    // Movement
    OverallMovement { weight: f32 },
    DistanceTravelled { weight: f32},
    TargetPosition { weight: f32, pos: (f32, f32)},
    PositionsVisited { weight: f32 },

    // Shape
    Shape { weight: f32 },

    // Basic Stuff
    Alive { weight: f32 },
}
impl FitnessEvaluator
{
    pub fn evaluate(&mut self, statistics: &Vec<FitnessStatistic>) -> (Instinct, f32)
    {
        debug!("Evaluating - {}", statistics.len());

        match *self
        {
            FitnessEvaluator::OverallMovement{ weight: w } => 
            {
                let i = Instinct::Nomadic;
                let v = statistics.iter().fold(0.0,
                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::Moved =>
                                                      {
                                                         accum += w;
                                                      },
                                                      _ => {}
                                                  }
                                               accum
                                               });
                debug!("Evaluated {} for {} due to Overall Movement", v, i);
                (i, v)
            },
            FitnessEvaluator::DistanceTravelled { weight: w } =>
            {
                let i = Instinct::Nomadic;
                let v = statistics.iter().fold(0.0,
                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::DistanceTravelled(dist) =>
                                                      {
                                                         accum += (w * dist as f32);
                                                      },
                                                      _ => {}
                                                  }
                                               accum
                                               });

                debug!("Evaluated {} for {} due to Distance Travelled", v, i);
                (i, v)
            },
            FitnessEvaluator::Shape { weight: w } =>
            {
                let i = Instinct::Hoarding;
                let v = statistics.iter().fold(0.0,
                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::TotalCells(c) =>
                                                      {
                                                         accum += w - ((10.0 - c as f32).abs());
                                                      },
                                                      _ => {}
                                                  }
                                                  accum
                                               });
                debug!("Evaluated {} for {} due to Shape", v, i);
                (i,v)
            },
            FitnessEvaluator::Alive { weight: w } =>
            {
                let i = Instinct::Basic;
                let v = statistics.iter().fold(w,

                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::Died(step, max) =>
                                                      {
                                                          accum *= ( step as f32 / max as f32 );
                                                      },
                                                      _ => {}
                                                  }
                                                  accum
                                               });
                debug!("Evaluated {} for {} due to Staying Alive", v, i);
                (i,v)
            },
            FitnessEvaluator::TargetPosition { weight: w, pos: target } =>
            {
                let i = Instinct::Basic;
                let v = statistics.iter().fold(0.0,
                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::FinalPosition(actual_x, actual_y) =>
                                                      {
                                                          let actual = ( (actual_x as f32) / 255.0,
                                                                         (actual_y as f32) / 255.0);
                                                          let dx = (target.0 - actual.0).abs();
                                                          let dy = (target.1 - actual.1).abs();
                                                          accum = w * (1.0 - dx)  +
                                                                  w * (1.0 - dy);
                                                          accum /= (dx + dy);
                                                      },
                                                      _ => {}
                                                  }
                                                  accum
                                               });

                debug!("Evaluated {} for {} due to Target Position {:?}", v, i, target);
                (i,v)
            },
            FitnessEvaluator::PositionsVisited { weight : w } =>
            {
                let i = Instinct::Nomadic;
                let mut already_counted = HashSet::new();
                let mut already_counted_x = HashSet::new();
                let mut already_counted_y = HashSet::new();
                let mut v = statistics.iter().fold(0.0,
                                               |mut accum, stat|
                                               {
                                                  match stat
                                                  {
                                                      &FitnessStatistic::PositionVisited( pos ) =>
                                                      {
                                                          if !already_counted.contains(&pos)
                                                          {
                                                              already_counted.insert(pos);
                                                              accum += w;
                                                          }
                                                          if !already_counted_x.contains(&pos.0)
                                                          {
                                                              already_counted_x.insert(pos.0);
                                                          }
                                                          if !already_counted_y.contains(&pos.1)
                                                          {
                                                              already_counted_y.insert(pos.1);
                                                          }
                                                      },
                                                      _ => {}
                                                  }
                                                  accum
                                               });

                if already_counted_x.len() > 1 && already_counted_y.len() > 1
                {
                    // If movement happened in X and Y give a
                    // bonus
                    v *= 1.1
                }
                debug!("Evaluated {} for {} due to Positions Visited", v, i);
                (i,v)
            }
        }
    }

    fn get_associated_instinct(&self) -> Instinct
    {
        match *self
        {
            FitnessEvaluator::OverallMovement   { weight: _ }   |
            FitnessEvaluator::DistanceTravelled { weight: _ }   |
            FitnessEvaluator::TargetPosition    { weight: _,
                                                  pos: _ } =>
            {
                Instinct::Nomadic
            },

            FitnessEvaluator::Shape { weight: _ } =>
            {
                Instinct::Hoarding
            },

            _ =>
            {
                Instinct::Basic
            }
        }
    }
}
impl Serializable for FitnessEvaluator
{
    fn serialize(&self, _: &mut SerializationCtx) -> Json
    {
        let mut json_obj = pmJsonObject::new();
        let w;
        let id;
        match *self
        {
            FitnessEvaluator::OverallMovement{ weight: weight } => 
            {
                w = weight;
                id = "overallmovement";
            },
            FitnessEvaluator::DistanceTravelled{ weight: weight } => 
            {
                w = weight;
                id = "distancetravelled";
            },
            FitnessEvaluator::TargetPosition{ weight: weight, pos: pos } =>
            {
                w = weight;
                id = "targetposition";

                let mut pos_json = pmJsonObject::new(); 
                pos_json.insert("x".to_owned(), pos.0.to_json());
                pos_json.insert("y".to_owned(), pos.1.to_json());

                json_obj.insert("Position".to_owned(), Json::Object(pos_json));
            },
            FitnessEvaluator::PositionsVisited{ weight: weight } => 
            {
                id = "positionsvisited";
                w = weight;
            },
            FitnessEvaluator::Shape{ weight: weight } => 
            {
                w = weight;
                id = "shape";
            },
            FitnessEvaluator::Alive{ weight: weight } => 
            {
                w = weight;
                id = "alive";
            },
        };
        json_obj.insert("EvaluatorId".to_owned(), id.to_json());
        json_obj.insert("Weight".to_owned(), w.to_json());
        Json::Object(json_obj)
    }
}
impl Deserializable for FitnessEvaluator
{
    fn new_from_json(json: &Json, _: &mut SerializationCtx) -> Option<FitnessEvaluator> 
    {
        match *json
        {
            Json::Object(ref json_obj) =>
            {
                let w = json_obj.get("Weight").unwrap().as_f64().unwrap() as f32;
                let fe = match json_obj.get("EvaluatorId").unwrap().as_string().unwrap()
                {
                    "overallmovement" => { FitnessEvaluator::OverallMovement{ weight: w }},
                    "distancetravelled" => { FitnessEvaluator::DistanceTravelled{ weight: w}},
                    "shape" => { FitnessEvaluator::Shape{ weight: w }},
                    "positionsvisited" => { FitnessEvaluator::PositionsVisited{ weight: w }},

                    "targetposition" =>
                    {
                        let pos_json = json_obj.get("Position").unwrap().as_object().unwrap();
                        FitnessEvaluator::TargetPosition{ weight: w,
                                                          pos: (pos_json.get("x").unwrap().as_f64().unwrap() as f32,
                                                                pos_json.get("y").unwrap().as_f64().unwrap() as f32) }


                    },
                    "alive" => { FitnessEvaluator::Alive{ weight: w }},
                    _ => { return None }
                };
                Some(fe)
            },
            _ => 
            {
                error!("Incorrect Type Passed for FitnessEvaluator");
                None
            }
        }
    }
}

pub struct PolyminiEvaluationCtx
{
    evaluators: Vec<FitnessEvaluator>,
    accumulator: PolyminiFitnessAccumulator,
    instinct_weights: HashMap<Instinct, f32>,

    accumulates_over: bool,
}
impl PolyminiEvaluationCtx
{
    pub fn new_from(evaluators: Vec<FitnessEvaluator>, accumulator: PolyminiFitnessAccumulator, instinct_weights: HashMap<Instinct, f32>,
                    accumulates_over: bool) -> PolyminiEvaluationCtx
    {
        PolyminiEvaluationCtx { evaluators: evaluators,
                                accumulator: accumulator,
                                instinct_weights: instinct_weights,
                                accumulates_over: accumulates_over }
    }

    pub fn evaluate(&mut self, statistics: &Vec<FitnessStatistic>)
    {
        debug!("EvaluationCtx::evaluate Before fold - {}", self.evaluators.len());
        self.evaluators.iter_mut().fold(&mut self.accumulator,
                                        |accum, ref mut evaluator|
                                        {
                                            debug!("In fold iteration");
                                            let v = evaluator.evaluate(statistics);
                                            accum.add(&v.0, v.1);
                                            accum
                                        });
        debug!("EvaluationCtx::evaluate After fold - {}", self.evaluators.len());
    }

    pub fn get_raw(&self) -> f32
    {
        self.calculate_fitness(&HashMap::new())
    }

    pub fn get_fitness(&self) -> f32
    {
        self.calculate_fitness(&self.instinct_weights)
    }

    fn calculate_fitness(&self, weights: &HashMap<Instinct,f32>) -> f32
    {
        let mut res = 0.0;
        for (instinct, score) in &self.accumulator.accumulated_by_instinct
        {
            res += (score * match weights.get(&instinct)
                            {
                                Some(v) => { *v },
                                None => { 1.0 }
                            });
        }
        debug!("EvaluationCtx::calculate_fitness: Accumulated {}", res);

        if res >= 0.0
        {
            res
        }
        else
        {
            0.0
        }
    }

    pub fn accumulates_over(&self) -> bool
    {
        self.accumulates_over
    }

    pub fn get_per_instinct(&self) -> &HashMap<Instinct, f32>
    {
        &self.accumulator.accumulated_by_instinct
    }
}

pub struct PolyminiFitnessAccumulator
{
    accumulated_by_instinct: HashMap<Instinct, f32>,
}
impl PolyminiFitnessAccumulator
{
    pub fn new(instincts: Vec<Instinct>) -> PolyminiFitnessAccumulator
    {
        let mut map = HashMap::new();

        assert!(instincts.len() > 0, "No instincts will yield no evolution");

        for i in &instincts
        {
            map.insert(*i, 0.0); 
        }

        PolyminiFitnessAccumulator { accumulated_by_instinct: map }
    }
    pub fn add(&mut self, instinct: &Instinct, v: f32)
    {
        let new_v;
        match self.accumulated_by_instinct.get(instinct)
        {
            Some(accum) => { new_v = accum + v; },
            None => { panic!("Incorrectly Initialized Accumulator found {} Instinct", instinct) }
        }

        debug!("Inserting {} for {}", new_v, instinct);
        self.accumulated_by_instinct.insert(*instinct, new_v);
    }
}

#[cfg(test)]
mod test
{
    use ::actuators::*;
    use ::instincts::*;
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn accumluator_test()
    {
        let mut accum = PolyminiFitnessAccumulator::new(vec![Instinct::Herding, 
                                                             Instinct::Hoarding,
                                                             Instinct::Predatory,
                                                             Instinct::Nomadic]);
        accum.add(&Instinct::Nomadic, 1.0);
        accum.add(&Instinct::Nomadic, 1.0);
        accum.add(&Instinct::Predatory, 1.0);
        
        let mut map = HashMap::new();
        map.insert(Instinct::Nomadic, 2.0);
        let eval_ctx = PolyminiEvaluationCtx { evaluators: vec![], accumulator: accum, accumulates_over: false, instinct_weights: map.clone() };

        assert_eq!(eval_ctx.get_raw(), 3.0);
        assert_eq!(eval_ctx.get_fitness(), 5.0);
    }
}
