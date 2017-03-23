extern crate polyminis_core;
#[macro_use]
extern crate log;
#[cfg(test)]

#[cfg(test)]
mod test
{
    extern crate env_logger;
    use polyminis_core::actuators::*;
    use polyminis_core::environment::*;
    use polyminis_core::evaluation::*;
    use polyminis_core::genetics::*;
    use polyminis_core::morphology::*;
    use polyminis_core::polymini::*;
    use polyminis_core::sensors::*;
    use polyminis_core::serialization::*;
    use polyminis_core::simulation::*;
    use polyminis_core::species::*;
    use polyminis_core::traits::*;

    use std::collections::{HashMap, HashSet};
    use std::time::{Duration, Instant};

    #[ignore]
    #[test]
    pub fn main_test()
    {
        let mut sim = Simulation::new(); 
        let _ = env_logger::init();

        let mut master_translation_table = HashMap::new();

        master_translation_table.insert( (TraitTier::TierI, 1), PolyminiTrait::PolyminiSimpleTrait(TraitTag::SpeedTrait));
        master_translation_table.insert( (TraitTier::TierI, 3), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveHorizontal));
        master_translation_table.insert( (TraitTier::TierI, 2), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveVertical));

        let mut active_table_1 = HashSet::new();
        active_table_1.insert( (TraitTier::TierI, 3) );
        active_table_1.insert( (TraitTier::TierI, 2) );
        active_table_1.insert( (TraitTier::TierI, 1) );

        let mut active_table_2 = HashSet::new();
        active_table_2.insert( (TraitTier::TierI, 3) );
        active_table_2.insert( (TraitTier::TierI, 2) );
        active_table_2.insert( (TraitTier::TierI, 1) );


        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1)];

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 2.5 },
                               FitnessEvaluator::DistanceTravelled { weight: 2.0 },
                               FitnessEvaluator::Shape { weight: 5.0 },
                               FitnessEvaluator::Alive { weight: 10.0 },
                               FitnessEvaluator::PositionsVisited { weight: 0.5 },
                               FitnessEvaluator::TargetPosition { weight: 15.0, pos: (1.0, 1.0) },
                               FitnessEvaluator::TargetPosition { weight: 15.0, pos: (1.0, 0.0) },
                               ];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);


        let mut env = Environment::new(2, default_sensors);

        let steps_per_epoch = 50;

        let cfg = PGAConfig { population_size: 50,
                              percentage_elitism: 0.2, percentage_mutation: 0.1, fitness_evaluators: evaluators,
                              genome_size: 8 };

        trace!("Creating Species");
        let ss = Species::new_from("Test Species".to_owned(), translation_table_species_1,
                                   &env.default_sensors, cfg,
                                   Box::new( | ctx: &mut PolyminiRandomCtx |
                                   {
                                        ( (ctx.gen_range(0.0, 100.0) as f32).floor(),
                                          (ctx.gen_range(0.0, 100.0) as f32).floor())
                                   }
                                   ));

        trace!("Adding Species");
        let mut epoch = SimulationEpoch::new_restartable(env, steps_per_epoch as usize, 1);
        epoch.add_species(ss);
        
        trace!("Swaping Epoch:");
        sim.swap_epoch(epoch);

        trace!("Running Epoch:");

        
        debug!("{}", sim.get_epoch()
                    .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));

        // TODO: Make this an easy to parameterize thing
        let total_epochs = 20;
        let mut serialization_ctx = SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG);
        for i in 0..total_epochs
        {
            let now = Instant::now();
            println!("Starting Epoch");
            loop 
            {
                debug!("Before Step:");
                if sim.step()
                {
                    break;
                }
                debug!("After Step: ");
                debug!("{}", sim.get_epoch()
                            .serialize(&mut serialization_ctx));


                for s in sim.get_epoch().get_species()
                {
                    println!("Best Individual of Species {} {}", s.get_name(),
                          s.get_best().serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DYNAMIC)));
                }

            }
            println!("After Epoch - {}s {}ms", now.elapsed().as_secs(), now.elapsed().subsec_nanos() / 1000000);
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx)); 


            sim.get_epoch_mut().evaluate_species(); 

            trace!("After Eval");
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx));

            for s in sim.get_epoch().get_species()
            {
                println!("{}", s.get_best().serialize(&mut serialization_ctx));
            }

            if i < total_epochs - 1
            {
                sim.advance_epoch();
                trace!("After Advancing Epoch");
                trace!("{}", sim.get_epoch()
                       .serialize(&mut serialization_ctx));
            }
        }

        for s in sim.get_epoch().get_species()
        {
            println!("{}", s.get_best().serialize(&mut serialization_ctx));
        }

        sim.get_epoch_mut().dump_species_random_ctx();
    }

    #[ignore]
    #[test]
    fn test_solo_run()
    {
        let mut sim = Simulation::new(); 
        let _ = env_logger::init();

        let mut master_translation_table = HashMap::new();

        master_translation_table.insert( (TraitTier::TierI, 8), PolyminiTrait::PolyminiSimpleTrait(TraitTag::SpeedTrait));
        master_translation_table.insert( (TraitTier::TierI, 7), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveHorizontal));
        master_translation_table.insert( (TraitTier::TierI, 6), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveVertical));

        let mut active_table_1 = HashSet::new();
        active_table_1.insert( (TraitTier::TierI, 8) );
        active_table_1.insert( (TraitTier::TierI, 7) );
        active_table_1.insert( (TraitTier::TierI, 6) );

        let mut active_table_2 = HashSet::new();
        active_table_2.insert( (TraitTier::TierI, 8) );
        active_table_2.insert( (TraitTier::TierI, 7) );
        active_table_2.insert( (TraitTier::TierI, 6) );


        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1)];

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 0.5 },
                               FitnessEvaluator::DistanceTravelled { weight: 0.5 },
                               FitnessEvaluator::Alive { weight: 15.0 },
                               FitnessEvaluator::Shape { weight: 8.0 },
                               FitnessEvaluator::PositionsVisited { weight: 3.5 },
                               ];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);


        let mut env = Environment::new(2, default_sensors);

        env.add_static_object( (0.0, 0.0),   (50, 1));
        env.add_static_object( (0.0, 0.0),   (1, 50));
        env.add_static_object( (49.0, 0.0),  (1, 50));
        env.add_static_object( (0.0, 49.0),  (50, 1));


        env.add_static_object( (20.0, 10.0),  (10, 30));
        env.add_static_object( (10.0, 20.0),  (30, 10));

        let mut ctx = PolyminiRandomCtx::from_seed([3,1,4,2], "Temp".to_owned());
        for i in 0..ctx.gen_range(0, 10)
        {
            env.add_static_object(((ctx.gen_range(0.0, 50.0) as f32).floor(), (ctx.gen_range(0.0, 50.0) as f32).floor()), (4, 4));
        }


        let steps_per_epoch = 50;

        let cfg = PGAConfig { population_size: 50,
                              percentage_elitism: 0.20, percentage_mutation: 0.3, fitness_evaluators: evaluators,
                              genome_size: 4 };

        let mut epoch = SimulationEpoch::new_restartable(env, steps_per_epoch as usize, 0);

        trace!("Creating Species");
        for tt in vec![translation_table_species_1, translation_table_species_2] 
        {
            let ss = Species::new_from("Test Species 1".to_owned(), tt,
                                       &epoch.get_environment().default_sensors, cfg.clone(),
                                       Box::new( | ctx: &mut PolyminiRandomCtx | {
                                           (11.0, 11.0)
                                       }));

            trace!("Adding Species");
            epoch.add_species(ss);
        }
               
        trace!("Swaping Epoch:");
        sim.swap_epoch(epoch);

        trace!("Running Epoch:");
 
        let cfg_2 = cfg.clone();
        //cfg_2.fitness_evaluators = vec![];

        // TODO: Make this an easy to parameterize thing
        let total_epochs = 50;
        let mut serialization_ctx = SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG);
        for i in 0..total_epochs
        {
            let now = Instant::now();
            println!("Starting Solo Run");
            {
                let new_env = sim.get_epoch().get_environment().clone();
                sim.get_epoch_mut().solo_run(&vec![
                                                    (new_env.clone(), cfg_2.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                       ( (ctx.gen_range(12.0, 18.0) as f32).floor(),
                                                         (ctx.gen_range(12.0, 18.0) as f32).floor())
                                                     })),
                                                    (new_env.clone(), cfg_2.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                       ( (ctx.gen_range(32.0, 38.0) as f32).floor(),
                                                         (ctx.gen_range(32.0, 38.0) as f32).floor())
                                                     })),
                                                     (new_env.clone(), cfg_2.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                       ( (ctx.gen_range(12.0, 18.0) as f32).floor(),
                                                         (ctx.gen_range(32.0, 38.0) as f32).floor())
                                                     })),
                                                     (new_env.clone(), cfg_2.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                       ( (ctx.gen_range(32.0, 38.0) as f32).floor(),
                                                         (ctx.gen_range(12.0, 18.0) as f32).floor())
                                                     })),
                                                     ]);
            }
            println!("After Solo Run- {}s {}ms", now.elapsed().as_secs(), now.elapsed().subsec_nanos() / 1000000);
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx)); 


            sim.get_epoch_mut().evaluate_species(); 

            trace!("After Eval");
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx));

            for s in sim.get_epoch().get_species()
            {
                println!("{}", s.get_best().serialize(&mut serialization_ctx));
            }

            if i < total_epochs - 1
            {
                sim.advance_epoch();
                trace!("After Advancing Epoch");
                trace!("{}", sim.get_epoch()
                       .serialize(&mut serialization_ctx));
            }
        }

        for s in sim.get_epoch().get_species()
        {
            println!("{}", s.serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB)));
        }

        sim.get_epoch_mut().dump_species_random_ctx();
        println!("{}", sim.get_epoch()
                    .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB)));
    }

    #[ignore]
    #[test]
    fn test_generate_seed_json()
    {
        let _ = env_logger::init();

        let mut master_translation_table = HashMap::new();

        master_translation_table.insert( (TraitTier::TierI, 8), PolyminiTrait::PolyminiSimpleTrait(TraitTag::SpeedTrait));
        master_translation_table.insert( (TraitTier::TierI, 7), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveHorizontal));
        master_translation_table.insert( (TraitTier::TierI, 6), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveVertical));

        let mut active_table_1 = HashSet::new();
        active_table_1.insert( (TraitTier::TierI, 8) );
        active_table_1.insert( (TraitTier::TierI, 7) );
        active_table_1.insert( (TraitTier::TierI, 6) );

        let mut active_table_2 = HashSet::new();
        active_table_2.insert( (TraitTier::TierI, 3) );
        active_table_2.insert( (TraitTier::TierI, 2) );
        active_table_2.insert( (TraitTier::TierI, 1) );


        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1)];

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 2.5 },
                               FitnessEvaluator::DistanceTravelled { weight: 2.0 },
                               FitnessEvaluator::Alive { weight: 8.0 },
                               FitnessEvaluator::PositionsVisited { weight: 0.5 },
                               FitnessEvaluator::TargetPosition { weight: 15.0, pos: (1.0, 1.0) },
                               FitnessEvaluator::TargetPosition { weight: 15.0, pos: (1.0, 0.0) },
                               ];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);


        let mut env = Environment::new(2, default_sensors);

        env.add_static_object( (0.0, 0.0),   (100, 1));
        env.add_static_object( (0.0, 0.0),   (1, 100));
        env.add_static_object( (99.0, 0.0),  (1, 100));
        env.add_static_object( (0.0, 99.0),  (100, 1));


        let steps_per_epoch = 50;

        let cfg = PGAConfig { population_size: 5,
                              percentage_elitism: 0.2, percentage_mutation: 0.1, fitness_evaluators: evaluators,
                              genome_size: 8 };

        trace!("Creating Species");
        let ss = Species::new_from("Test Species".to_owned(), translation_table_species_1,
                                   &env.default_sensors, cfg,
                                   Box::new( | ctx: &mut PolyminiRandomCtx |
                                   {
                                        ( (ctx.gen_range(0.0, 100.0) as f32).floor(),
                                          (ctx.gen_range(0.0, 100.0) as f32).floor())
                                   }
                                   ));

        trace!("Adding Species");
        let mut epoch = SimulationEpoch::new_restartable(env, steps_per_epoch as usize, 1);

        let mut ser_ctx = SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB);
        trace!( "{}", epoch.serialize(&mut ser_ctx).to_string());
        trace!( "{}", ss.serialize(&mut ser_ctx).to_string()); 

        let mut mtt_json = pmJsonArray::new();
        for ((ttier, nid), pm_trait) in master_translation_table
        {
            let mut entry = pmJsonObject::new();
            entry.insert("Tier".to_owned(), ttier.serialize(&mut ser_ctx)); 
            entry.insert("TID".to_owned(), nid.to_json()); 
            entry.insert("Trait".to_owned(), pm_trait.to_string().to_lowercase().to_json());
            mtt_json.push(Json::Object(entry));
        }

        trace!("{}", Json::Array(mtt_json).to_string());
    }
}

