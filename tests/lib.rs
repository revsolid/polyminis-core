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

        let steps_per_epoch = 100;

        let cfg = PGAConfig { population_size: 50,
                              percentage_elitism: 0.2, percentage_mutation: 0.1, fitness_evaluators: evaluators, accumulates_over: false,
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
        active_table_1.insert( (TraitTier::TierI, 7) );
        active_table_1.insert( (TraitTier::TierI, 6) );

        let mut active_table_2 = HashSet::new();
        active_table_2.insert( (TraitTier::TierI, 7) );
        active_table_2.insert( (TraitTier::TierI, 6) );

        let mut active_table_3 = HashSet::new();
        active_table_3.insert( (TraitTier::TierI, 7) );
        active_table_3.insert( (TraitTier::TierI, 6) );

        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1),
                                    Sensor::new(SensorTag::TimeGlobal, 1),
                                    Sensor::new(SensorTag::TimeSubStep, 1),
                                    ];

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 0.75 },
                               FitnessEvaluator::DistanceTravelled { weight: 0.75 },
                               FitnessEvaluator::Alive { weight: 1.0 },
                               FitnessEvaluator::Shape { weight: 2.0 },
                               FitnessEvaluator::PositionsVisited { weight: 3.5 },
                               ];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);
        let translation_table_species_3 = TranslationTable::new_from(&master_translation_table, &active_table_3);



        let mut env = Environment::new(3, default_sensors.clone());
        env.add_static_object( (0.0, 0.0),   (50, 1), false);
        env.add_static_object( (0.0, 0.0),   (1, 50), false);
        env.add_static_object( (49.0, 0.0),  (1, 50), false);
        env.add_static_object( (0.0, 49.0),  (50, 1), false);


        env.add_static_object( (25.0, 10.0),  (1, 30), false);
        env.add_static_object( (10.0, 25.0),  (30, 1), false);

        let steps_per_epoch = 50;

        let cfg = PGAConfig { population_size: 60,
                              percentage_elitism: 0.2, percentage_mutation: 0.35, fitness_evaluators: evaluators, accumulates_over: false,
                              genome_size: 4 };

        let mut empty = Environment::new_with_dimensions(3, default_sensors.clone(), (5000.0, 5000.0));
        let mut epoch = SimulationEpoch::new_restartable(empty.clone(), steps_per_epoch as usize, 2);

        trace!("Creating Species");
        let mut i = 0;
        for tt in vec![translation_table_species_1, translation_table_species_2, translation_table_species_3]
        {
            let ss = Species::new_from(format!("Test Species {}", i), tt,
                                       &epoch.get_environment().default_sensors, cfg.clone(),
                                       Box::new( | ctx: &mut PolyminiRandomCtx | {
                                               ( (ctx.gen_range(5.0, 4950.0) as f32).floor(),
                                                 (ctx.gen_range(5.0, 4950.0) as f32).floor())
                                       }));

            trace!("Adding Species");
            epoch.add_species(ss);
            i += 1;
        }
               
        trace!("Swaping Epoch:");
        sim.swap_epoch(epoch);

        trace!("Running Epoch:");
 
        let mut cfg_2 = cfg.clone();

        let mut env2 = Environment::new(3, default_sensors.clone());
        env2.add_static_object( (0.0, 0.0),   (50, 1), false);
        env2.add_static_object( (0.0, 0.0),   (1, 50), false);
        env2.add_static_object( (50.0, 0.0),  (1, 50), false);
        env2.add_static_object( (0.0, 50.0),  (50, 1), false);

        env2.add_static_object( (15.0, 0.0),  (1, 30), false);
        env2.add_static_object( (35.0, 0.0),  (1, 30), false);
        env2.add_static_object( (25.0, 20.0), (1, 30), false);

        env2.add_static_object( (5.0, 10.0),  (10, 1), false);
        env2.add_static_object( (0.0, 30.0),  (10, 1), false);
        env2.add_static_object( (35.0, 10.0),  (10, 1), false);
        env2.add_static_object( (40.0, 30.0),  (10, 1), false);

        let evals_2 = vec![ FitnessEvaluator::OverallMovement { weight: 0.75 },
                            FitnessEvaluator::DistanceTravelled { weight: 0.75 },
                            FitnessEvaluator::Alive { weight: 1.0 },
                            FitnessEvaluator::Shape { weight: 2.0 },
                            FitnessEvaluator::PositionsVisited { weight: 3.5 },
                          ];
        cfg_2.fitness_evaluators = evals_2;


        let mut env3 = Environment::new(3, default_sensors.clone());
        env3.add_static_object( (0.0, 0.0),   (50, 1), false);
        env3.add_static_object( (0.0, 0.0),   (1, 50), false);
        env3.add_static_object( (50.0, 0.0),  (1, 50), false);
        env3.add_static_object( (0.0, 50.0),  (50, 1), false);

        env3.add_static_object( (15.0, 20.0),  (1, 30), false);
        env3.add_static_object( (35.0, 20.0),  (1, 30), false);
        env3.add_static_object( (25.0, 0.0), (1, 30), false);

        env3.add_static_object( (0.0, 20.0),  (10, 1), false);
        env3.add_static_object( (5.0, 40.0),  (10, 1), false);
        env3.add_static_object( (40.0, 20.0),  (10, 1), false);
        env3.add_static_object( (35.0, 40.0),  (10, 1), false);

        let cfg_3 = cfg_2.clone();

        let mut env4 = Environment::new(3, default_sensors.clone());
        env4.add_static_object( (0.0, 0.0),   (50, 1), false);
        env4.add_static_object( (0.0, 0.0),   (1, 50), false);
        env4.add_static_object( (50.0, 0.0),  (1, 50), false);
        env4.add_static_object( (0.0, 50.0),  (50, 1), false);

        env4.add_static_object( (21.0, 21.0),  (1, 6), false);
        env4.add_static_object( (27.0, 21.0),  (1, 6), false);

        env4.add_static_object( (18.0, 31.0),  (12, 1), false);
        env4.add_static_object( (18.0, 17.0),  (12, 1), false);

        env4.add_static_object( (14.0, 14.0),  (12, 1), false);
        env4.add_static_object( (35.0, 14.0),  (12, 1), false);

        let mut cfg_4 = cfg_2.clone();
        let evals_4 = vec![ FitnessEvaluator::OverallMovement { weight: 0.75 },
                            FitnessEvaluator::DistanceTravelled { weight: 1.5 },
                            FitnessEvaluator::Alive { weight: 1.0 },
                            FitnessEvaluator::Shape { weight: 5.0 },
                            FitnessEvaluator::PositionsVisited { weight: 2.5 },
                          ];
        cfg_4.fitness_evaluators = evals_4;


        // TODO: Make this an easy to parameterize thing
        let total_epochs = 1;
        let mut serialization_ctx = SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG);
        let outer_now = Instant::now();
        println!("Starting {} Epochs", total_epochs);
        for i in 0..total_epochs
        {
            let now = Instant::now();
            println!("Starting Solo Run");
            {
                sim.get_epoch_mut().solo_run(&vec![
                                                    (env.clone(), cfg.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (16.0, 16.0)
                                                       // ( (ctx.gen_range(15.0, 20.0) as f32).floor(),
                                                       // (ctx.gen_range(15.0, 20.0) as f32).floor())
                                                     })),
                                                    (env.clone(), cfg.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (31.0, 31.0)
                                                       // ( (ctx.gen_range(30.0, 35.0) as f32).floor(),
                                                       //  (ctx.gen_range(30.0, 35.0) as f32).floor())
                                                     })),
                                                    (env.clone(), cfg.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (16.0, 31.0)
                                                       // ( (ctx.gen_range(15.0, 20.0) as f32).floor(),
                                                       // (ctx.gen_range(15.0, 20.0) as f32).floor())
                                                     })),
                                                    (env.clone(), cfg.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (31.0, 16.0)
                                                       // ( (ctx.gen_range(15.0, 20.0) as f32).floor(),
                                                       // (ctx.gen_range(15.0, 20.0) as f32).floor())
                                                     })),


                    
                                                     (env2.clone(), cfg_2.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                        (8.0, 4.0)
                                                       //( (ctx.gen_range(7.0, 11.0) as f32).floor(),
                                                       //(ctx.gen_range(7.0, 11.0) as f32).floor())
                                                     })),
                                                     (env3.clone(), cfg_3.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (40.0, 45.0)
                                                       //( (ctx.gen_range(38.0, 42.0) as f32).floor(),
                                                       //(ctx.gen_range(38.0, 42.0) as f32).floor())
                                                     })),

                                                     (env4.clone(), cfg_4.clone(),
                                                     Box::new( | ctx: &mut PolyminiRandomCtx |
                                                     {
                                                         (25.0, 25.0)
                                                     }))
                                                     ]);
            }
            println!("After Solo Run- {}s {}ms", now.elapsed().as_secs(), now.elapsed().subsec_nanos() / 1000000);

            for s in sim.get_epoch().get_species()
            {
                println!("{}", s.get_best().serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_STATS)));
            }

            if i < total_epochs - 1
            {
                let adv_now = Instant::now();
                sim.advance_epoch();
                println!("After Advancing Epoch {}s", adv_now.elapsed().as_secs());
            }
        }


        for s in sim.get_epoch().get_species()
        {
            println!("{}", s.serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB)));
        }

        println!("{}", sim.get_epoch()
                    .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DB)));

        println!("After {} Epochs - {}s", total_epochs, outer_now.elapsed().as_secs());
    }


    #[ignore]
    #[test]
    fn test_walls_bug()
    {
        let mut sim = Simulation::new(); 
        let _ = env_logger::init();

        let mut master_translation_table = HashMap::new();
        master_translation_table.insert( (TraitTier::TierI, 8), PolyminiTrait::PolyminiSimpleTrait(TraitTag::SpeedTrait));
        master_translation_table.insert( (TraitTier::TierI, 7), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveHorizontal));
        master_translation_table.insert( (TraitTier::TierI, 6), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveVertical));

        let mut active_table_3 = HashSet::new();
        active_table_3.insert( (TraitTier::TierI, 8) );
        active_table_3.insert( (TraitTier::TierI, 7) );
        active_table_3.insert( (TraitTier::TierI, 6) );

        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1)];

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 0.75 },
                               FitnessEvaluator::DistanceTravelled { weight: 0.75 },
                               FitnessEvaluator::Alive { weight: 1.0 },
                               FitnessEvaluator::Shape { weight: 2.0 },
                               FitnessEvaluator::PositionsVisited { weight: 3.5 },
                               ];

        let translation_table_species_3 = TranslationTable::new_from(&master_translation_table, &active_table_3);

        let steps_per_epoch = 50;
        let cfg = PGAConfig { population_size: 60,
                              percentage_elitism: 0.2, percentage_mutation: 0.35, fitness_evaluators: evaluators, accumulates_over: false,
                              genome_size: 4 };


        let mut env = Environment::new(3, default_sensors.clone());

        env.add_static_object( (0.0, 0.0),   (50, 1), false);
        env.add_static_object( (0.0, 0.0),   (1, 50), false);
        env.add_static_object( (49.0, 0.0),  (1, 50), false);
        env.add_static_object( (0.0, 49.0),  (50, 1), false);


        env.add_static_object( (25.0, 10.0),  (1, 30), false);
        env.add_static_object( (10.0, 25.0),  (30, 1), false);

        let mut epoch = SimulationEpoch::new_restartable(env, steps_per_epoch as usize, 2);

        let json = Json::from_str("
       {\"GAConfiguration\":{\"FitnessEvaluators\":[{\"EvaluatorId\":\"overallmovement\",\"Weight\":0.75},{\"EvaluatorId\":\"distancetravelled\",\"Weight\":0.75},{\"EvaluatorId\":\"alive\",\"Weight\":1.0},{\"EvaluatorId\":\"shape\",\"Weight\":2.0},{\"EvaluatorId\":\"positionsvisited\",\"Weight\":3.5}],\"GenomeSize\":4,\"PercentageElitism\":0.20000000298023224,\"PercentageMutation\":0.3499999940395355,\"PopulationSize\":60},\"Individuals\":[{\"Control\":{\"Hidden\":1,\"HiddenToOutput\":{\"Biases\":[-0.003571152687072754,-0.003571152687072754,-0.003571152687072754],\"Coefficients\":[-0.03838503360748291,-0.03838503360748291,0.012095451354980469],\"Inputs\":1,\"Outputs\":3},\"InToHidden\":{\"Biases\":[-0.4326575994491577],\"Coefficients\":[0.28510963916778564,0.8921076059341431,0.5872339010238647,0.3140263557434082],\"Inputs\":4,\"Outputs\":1},\"Input\":4,\"Output\":3},\"Fitness\":469.70001220703125,\"Morphology\":{\"Chromosome\":[[57,22,7,234],[222,42,229,45],[92,52,6,198],[53,108,6,198],[52,54,93,177],[26,53,190,48],[62,28,112,177],[93,20,27,54],[88,7,7,11],[193,58,106,241],[113,104,85,231]]},\"Raw\":469.70001220703125,\"Speed\":1},{\"Control\":{\"Hidden\":1,\"HiddenToOutput\":{\"Biases\":[-0.003571152687072754,-0.003571152687072754,-0.003571152687072754],\"Coefficients\":[-0.03838503360748291,-0.03838503360748291,0.012095451354980469],\"Inputs\":1,\"Outputs\":3},\"InToHidden\":{\"Biases\":[-0.4326575994491577],\"Coefficients\":[0.28510963916778564,0.8921076059341431,0.5872339010238647,0.3140263557434082],\"Inputs\":4,\"Outputs\":1},\"Input\":4,\"Output\":3},\"Fitness\":469.70001220703125,\"Morphology\":{\"Chromosome\":[[57,22,7,234],[222,42,229,45],[92,52,6,198],[53,108,6,198],[52,54,93,177],[26,53,190,48],[62,28,112,177],[93,20,27,54],[88,7,7,11],[193,58,106,241],[113,104,85,231]]},\"Raw\":469.70001220703125,\"Speed\":1}], \"Percentage\":0.0,\"SpeciesName\":\"Test Species 2\",\"TranslationTable\":[{\"Number\":7,\"Tier\":\"TierI\"},{\"Number\":6,\"Tier\":\"TierI\"},{\"Number\":8,\"Tier\":\"TierI\"}], \"InstinctWeights\":{}} 
        ").ok().unwrap(); 
        let test_spec = Species::new_from_json(&json, &default_sensors.clone(),
                                               Box::new( | _ : &mut PolyminiRandomCtx | { (0.0, 0.0) } ),
                                               &master_translation_table, None).unwrap();

        epoch.add_species(test_spec);
        sim.swap_epoch(epoch);
        let mut serialization_ctx = SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG);
        {
            let now = Instant::now();
            for i in 0..1
            {
                println!("Starting Solo Run");
                {
                    let new_env = sim.get_epoch().get_environment().clone();
                    sim.get_epoch_mut().solo_run(&vec![
                                                        (new_env.clone(), cfg.clone(),
                                                         Box::new( | ctx: &mut PolyminiRandomCtx |
                                                         {
                                                             (31.0, 31.0)
                                                         })),
                                                         
                                                        (new_env.clone(), cfg.clone(),
                                                         Box::new( | ctx: &mut PolyminiRandomCtx |
                                                         {
                                                             (31.0, 31.0)
                                                         })),
                                                         
                                                        (new_env.clone(), cfg.clone(),
                                                         Box::new( | ctx: &mut PolyminiRandomCtx |
                                                         {
                                                             (31.0, 31.0)
                                                         })),
                                                         ]);
                }
            }
        }

        for s in sim.get_epoch().get_species()
        {
            println!("{}", s.get_best().serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_STATS)));
        }
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

        env.add_static_object( (0.0, 0.0),   (100, 1), false);
        env.add_static_object( (0.0, 0.0),   (1, 100), false);
        env.add_static_object( (99.0, 0.0),  (1, 100), false);
        env.add_static_object( (0.0, 99.0),  (100, 1), false);


        let steps_per_epoch = 50;

        let cfg = PGAConfig { population_size: 5,
                              percentage_elitism: 0.2, percentage_mutation: 0.1, fitness_evaluators: evaluators, accumulates_over: false,
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

