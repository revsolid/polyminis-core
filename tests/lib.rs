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

        let mut active_table_2 = HashSet::new();
        active_table_2.insert( (TraitTier::TierI, 3) );
        active_table_2.insert( (TraitTier::TierI, 2) );
        active_table_2.insert( (TraitTier::TierI, 1) );


        let default_sensors = vec![ Sensor::new(SensorTag::PositionX, 1),
                                    Sensor::new(SensorTag::PositionY, 1),
                                    Sensor::new(SensorTag::Orientation, 1),
                                    Sensor::new(SensorTag::LastMoveSucceded, 1)];

        let evaluators = vec![ FitnessEvaluator::OverallMovement,
                               FitnessEvaluator::DistanceTravelled ];

        let translation_table_species_1 = TranslationTable::new_from(&master_translation_table, &active_table_1);
        let translation_table_species_2 = TranslationTable::new_from(&master_translation_table, &active_table_2);


        let env = Environment::new(2, default_sensors);

        let gens_per_epoch = 100;

        let cfg = PGAConfig { max_generations: gens_per_epoch, population_size: 10,
                              percentage_elitism: 0.2, fitness_evaluators: evaluators };

        info!("Creating Species");
        let ss = Species::new_from("Test Species".to_owned(), translation_table_species_1,
                                   &env.default_sensors, cfg);

        info!("Adding Species");
        let mut epoch = SimulationEpoch::new_with(env, gens_per_epoch as usize);
        epoch.add_species(ss);
        
        info!("Swaping Species:");
        sim.swap_epoch(epoch);

        info!("Running Epoch:");

        
        debug!("{}", sim.get_epoch()
                    .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));

        for i in 0..5
        {
            loop 
            {
                info!("Before Step:");
                if sim.step()
                {
                    break;
                }
                info!("After Step:");
                debug!("{}", sim.get_epoch()
                            .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));
            }
            info!("After Epoch");
            info!("{}", sim.get_epoch()
                        .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));


            sim.get_epoch_mut().evaluate_species(); 

            info!("After Eval");
            info!("{}", sim.get_epoch()
                        .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));

            sim.advance_epoch();

            info!("After Advancing Epoch");
            info!("{}", sim.get_epoch()
                        .serialize(&mut SerializationCtx::new_from_flags(PolyminiSerializationFlags::PM_SF_DEBUG)));
        }

        sim.get_epoch_mut().dump_species_random_ctx();
    }
}
