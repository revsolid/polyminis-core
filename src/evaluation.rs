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

    Died,
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
                                                         accum += 10.0 - (10.0 - c as f32).abs();
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
                                                      &FitnessStatistic::Died =>
                                                      {
                                                          accum = 0.0;
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
                                                          accum = w * dx  +
                                                                  w * dy;
                                                          accum /= (dx * dx + dy * dy);
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
                let v = statistics.iter().fold(0.0,
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
                                                      },
                                                      _ => {}
                                                  }
                                                  accum
                                               });
                debug!("Evaluated {} for {} due to Positions Visited", v, i);
                (i,v)
            }
        }
    }
}

pub struct PolyminiEvaluationCtx
{
    evaluators: Vec<FitnessEvaluator>,
    accumulator: PolyminiFitnessAccumulator,
}
impl PolyminiEvaluationCtx
{
    pub fn new() -> PolyminiEvaluationCtx
    {
        PolyminiEvaluationCtx { evaluators: vec![],
                                accumulator: PolyminiFitnessAccumulator::new(vec![ Instinct::Basic ]) }
    }

    pub fn new_from(evaluators: Vec<FitnessEvaluator>, accumulator: PolyminiFitnessAccumulator) -> PolyminiEvaluationCtx
    {
        PolyminiEvaluationCtx { evaluators: evaluators,
                                accumulator: accumulator }
    }

    pub fn evaluate(&mut self, statistics: &Vec<FitnessStatistic>)
    {
        debug!("Before fold - {}", self.evaluators.len());
        self.evaluators.iter_mut().fold(&mut self.accumulator,
                                        |accum, ref mut evaluator|
                                        {
                                            debug!("In fold iteration");
                                            let v = evaluator.evaluate(statistics);
                                            accum.add(&v.0, v.1);
                                            accum
                                        });
        debug!("After fold - {}", self.evaluators.len());
    }

    pub fn get_raw(&self) -> f32
    {
        self.get_fitness(&HashMap::new())
    }

    pub fn get_fitness(&self, weights: &HashMap<Instinct, f32>) -> f32
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
        debug!("Accumulated {}", res);

        if res >= 0.0
        {
            res
        }
        else
        {
            0.0
        }
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
        
        let eval_ctx = PolyminiEvaluationCtx { evaluators: vec![], accumulator: accum };

        assert_eq!(eval_ctx.get_raw(), 3.0);
        let mut map = HashMap::new();
        map.insert(Instinct::Nomadic, 2.0);
        assert_eq!(eval_ctx.get_fitness(&map), 5.0);
    }
}
