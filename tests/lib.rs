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

    #[test]
    pub fn main_test()
    {
        let mut sim = Simulation::new(); 
        let _ = env_logger::init();

        let mut master_translation_table = HashMap::new();

        master_translation_table.insert( (TraitTier::TierI, 1), PolyminiTrait::PolyminiSimpleTrait(PolyminiSimpleTrait::SpeedTrait));
        master_translation_table.insert( (TraitTier::TierI, 2), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveHorizontal));
        master_translation_table.insert( (TraitTier::TierI, 3), PolyminiTrait::PolyminiActuator(ActuatorTag::MoveVertical));

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

        let evaluators = vec![ FitnessEvaluator::OverallMovement { weight: 0.5 },
                               FitnessEvaluator::DistanceTravelled { weight: 1.0 },
                               FitnessEvaluator::Shape { weight: 5.0 },
                               FitnessEvaluator::Alive { weight: 10.0 },
                               FitnessEvaluator::TargetPosition { weight: 15.0, pos: (1.0, 1.0) }];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);


        let mut env = Environment::new(2, default_sensors);

        env.add_static_object( (0.0, 0.0),   (100, 1));
        env.add_static_object( (0.0, 0.0),   (1, 100));
        env.add_static_object( (99.0, 0.0),  (1, 100));
        env.add_static_object( (0.0, 99.0),  (100, 1));


        let gens_per_epoch = 50;

        let cfg = PGAConfig { max_generations: gens_per_epoch, population_size: 50,
                              percentage_elitism: 0.2, percentage_mutation: 0.1, fitness_evaluators: evaluators, genome_size: 8 };

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
        let mut epoch = SimulationEpoch::new_restartable(env, gens_per_epoch as usize, 1);
        epoch.add_species(ss);
        
        trace!("Swaping Species:");
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
            info!("Starting Epoch");
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
                    info!("Best Individual of Species {} {}", s.get_name(),
                          s.get_best().serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DYNAMIC)));
                }

            }
            info!("After Epoch - {}s {}ms", now.elapsed().as_secs(), now.elapsed().subsec_nanos() / 1000000);
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx)); 


            sim.get_epoch_mut().evaluate_species(); 

            trace!("After Eval");
            trace!("{}", sim.get_epoch()
                        .serialize(&mut serialization_ctx));

            for s in sim.get_epoch().get_species()
            {
                info!("{}", s.get_best().serialize(&mut serialization_ctx));
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
            info!("{}", s.get_best().serialize(&mut serialization_ctx));
        }

        sim.get_epoch_mut().dump_species_random_ctx();
    }
}
