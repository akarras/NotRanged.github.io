use crate::actions::Action;
use crate::xiv_model::{State, Synth};
use genevo::ga::genetic_algorithm;
use genevo::operator::prelude::*;
use genevo::population::{
    ValueEncodedGenomeBuilder,
};
use genevo::prelude::*;
use genevo::prelude::{
    simulate, FitnessFunction, GenerationLimit, Simulation, SimulationBuilder,
};
use genevo::simulation::simulator::Simulator;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::{Display, Formatter};

// genotype, usize where index matches available action
type CrafterActions = Vec<usize>;

#[derive(Debug)]
struct SynthResult {
    // progress, (actual, recipe required)
    progress: (u32, u32),
    // quality, (actual, recipe required)
    quality: (u32, u32),
    // cp, (actual, cp required)
    cp: (u32, u32), // possibly add some time duration step?
}

impl SynthResult {
    fn new(synth: &Synth, state: &State) -> Self {
        Self {
            progress: (state.progress_state as u32, synth.recipe.difficulty),
            quality: (state.quality_gain as u32, synth.recipe.max_quality),
            cp: (state.cp_state as u32, synth.crafter.craft_points)
        }
    }
}

impl Display for SynthResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (p1, p2) = self.progress;
        let (q1, q2) = self.quality;
        let (cp1, cp2) = self.cp;
        write!(f, "p: {}/{}\nq: {}/{}\ncp: {}/{}", p1, p2, q1, q2, cp1, cp2)
    }
}

impl FitnessFunction<CrafterActions, u32> for Synth {
    fn fitness_of(&self, actions: &CrafterActions) -> u32 {
        let mut state: State = self.into();
        for action_index in actions {
            if *action_index == 0 {
                break;
            }
            let action = self.crafter.actions.get(*action_index - 1).unwrap();
            let tmp_state = state.add_action(*action);
            let violations = tmp_state.check_violations();
            if violations.is_okay() {
                state = tmp_state;
            } else {
                //println!("{:?}", violations);
                return 0; // invalid state, 0 fitness
            }
        }
        //println!("{:?}", state.durability_state);
        let over_progress = (state.progress_state - self.recipe.difficulty as i32).abs();
        (state.cp_state.max(self.recipe.max_quality as i32) + state.progress_state.max(self.recipe.difficulty as i32) - over_progress) as u32
    }

    fn average(&self, a: &[u32]) -> u32 {
        a.iter().sum::<u32>() / a.len() as u32
    }

    fn highest_possible_fitness(&self) -> u32 {
        self.recipe.difficulty + self.recipe.max_quality
    }

    fn lowest_possible_fitness(&self) -> u32 {
        0
    }
}

struct CrafterGenoType {
    abilities: Vec<Action>,
}

struct CrafterFitness;

#[wasm_bindgen]
pub struct CraftSimulator {
    // oh god this type is so long.
    // pub(crate) synth: SynthSim,
    pub(crate) sim: Simulator<
        GeneticAlgorithm<
            CrafterActions,
            u32,
            Synth,
            MaximizeSelector,
            SinglePointCrossBreeder,
            RandomValueMutator<CrafterActions>,
            ElitistReinserter<CrafterActions, u32, Synth>,
        >,
        GenerationLimit,
    >,
}

impl CraftSimulator {
    pub fn new(synth: Synth, max_length: usize, population_size: usize) -> Self {
        let number_of_available_actions = synth.crafter.actions.len();
        let initial_population: Population<CrafterActions> = build_population()
            .with_genome_builder(ValueEncodedGenomeBuilder::new(
                50,
                0, // define 0 as no operation, end of sequence
                number_of_available_actions + 1, // 1 is our real first ability
            ))
            .of_size(population_size)
            .uniform_at_random();
        let sim = simulate(
            genetic_algorithm()
                .with_evaluation(synth.clone())
                .with_selection(MaximizeSelector::new(0.85, 12))
                .with_crossover(SinglePointCrossBreeder::new())
                .with_mutation(RandomValueMutator::new(0.2, 0, number_of_available_actions))
                .with_reinsertion(ElitistReinserter::new(synth, false, 0.85))
                .with_initial_population(initial_population)
                .build(),
        )
        .until(GenerationLimit::new(750))
        .build();

        Self { sim }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, PartialOrd, Eq, Ord)]
enum SimStep {
    Complete(CrafterActions),
    Working(CrafterActions),
    Error,
}

#[wasm_bindgen]
impl CraftSimulator {
    fn next(&mut self) -> SimStep {
        match self.sim.step() {
            Ok(ok) => match ok {
                SimResult::Intermediate(a) => {
                    eprintln!("{:?}", a.result.best_solution.solution);

                    SimStep::Working(a.result.best_solution.solution.genome)
                }
                SimResult::Final(a, b, c, d) => SimStep::Complete(a.result.best_solution.solution.genome),
            },
            Err(_) => SimStep::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Action;
    use crate::simulator::{CraftSimulator, SimStep};
    use crate::xiv_model::{Crafter, Recipe, Synth};

    #[test]
    fn test_basic_synth() {
        let recipe = Recipe {
            base_level: 1,
            level: 1,
            difficulty: 100,
            durability: 60,
            start_quality: 0,
            max_quality: 100,
            suggested_craftsmanship: 1,
            suggested_control: 1,
            progress_divider: 1.0,
            progress_modifier: None,
            quality_divider: 1.0,
            quality_modifier: None,
            stars: false,
        };
        let crafter = Crafter {
            cls: 10,
            craftsmanship: 20,
            control: 20,
            craft_points: 10,
            level: 10,
            specialist: 0,
            actions: vec![Action::BasicSynth, Action::StandardTouch],
        };
        let synth = Synth {
            crafter,
            recipe,
            max_trick_uses: 10,
            reliability_index: 1,
            max_length: 50,
            solver_vars: Default::default(),
        };
        let mut sim = CraftSimulator::new(synth, 50, 10000);
        let sim_result = sim.next();
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        eprintln!("{:?}", sim.next());
        assert!(false);
    }
}
