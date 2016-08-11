mod Functionalus
{
    use rust_monster::ga::ga_core::*;
    use std::any::Any;
    struct PolyminiGenome;

    trait Controller
    {
        fn register_sensor(&self);
        fn register_actuator(&self);
        fn sense(&self);
        fn think(&self);
        fn act(&self);
    }

    trait PhysicsManager
    {
        fn register_object(&self);
    }

    trait MorphologyManager
    {
        fn init_from_genome(&self, _:&PolyminiGenome);
        //init_from_genome(.., .., MorphologyOptions);
    }

    struct Polymini<'a>
    {
        morph_mgr: &'a mut MorphologyManager,
        phy_mgr: &'a mut PhysicsManager,
        controller: &'a mut Controller,
    }

    impl<'a> Polymini<'a>
    {
        fn new(mm: &'a mut MorphologyManager,
               pm: &'a mut PhysicsManager,
               ctr: &'a mut Controller) -> Polymini<'a>
        {
            Polymini { morph_mgr: mm,
                       phy_mgr: pm,
                       controller: ctr }
        }
    }

    /*
    impl<'a> GAIndividual for Polymini<'a>
    {
        fn evaluate(&mut self) -> f32 { 0.0 }
        fn crossover(&self, _: &Polymini<'a>) -> Box<Polymini<'a>>
        { 
            Box::new(Polymini::)
        }
        fn mutate(&mut self, _: f32) {}
        fn fitness(&self) -> f32 { 0.0 }
        fn set_fitness(&mut self, _:f32) { }
        fn raw(&self) -> f32 { 0.0 }
    }
    */
}

mod Simulation
{
    enum Phases
    {
        // Sense
        // Think
        // Act
        // Consequence (?)
        // Env
    }

    struct PolyminiSpecies
    {
        /*
         * population: GAPopulation<Polymini>
         * genetic_algorithm: SimpleGA<Polymini>
         */
    }

    struct PolyminiSimulation
    {
        species: Vec<PolyminiSpecies>,
    }

    impl PolyminiSimulation
    {
        fn setup(&self){}
        fn step(&self){}
    }
}
