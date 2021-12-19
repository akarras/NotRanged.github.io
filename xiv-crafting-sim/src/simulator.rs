use crate::actions::Action;
use crate::xiv_model::{State, Synth, Violations, Condition};
use genevo::ga::genetic_algorithm;
use genevo::operator::prelude::*;
use genevo::population::ValueEncodedGenomeBuilder;
use genevo::prelude::*;
use genevo::prelude::{simulate, FitnessFunction, GenerationLimit, Simulation, SimulationBuilder};
use genevo::simulation::simulator::Simulator;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsValue};
use serde_json::value::Value;

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
            cp: (state.cp_state as u32, synth.crafter.craft_points),
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

trait CalcState {
    fn calculate_final_state(&self, synth: &Synth) -> State;

    fn get_actions_list(&self, synth: &Synth) -> Vec<Action>;

    fn get_final_actions_list(&self, synth: &Synth) ->(State, Vec<Action>);
}

impl CalcState for CrafterActions {
    fn calculate_final_state(&self, synth: &Synth) -> State {
        let mut state: State = synth.into();
        let actions: Vec<Action> = self.get_actions_list(synth);

        for action in actions {
            let tmp_state = state.add_action(action);
            let violations = tmp_state.check_violations();
            if violations.progress_ok {
                return tmp_state
            }
            if !violations.durability_ok {
                return tmp_state // bad durability, no point proceeding
            }
            if !violations.cp_ok {
                return state
            }
            state = tmp_state;
        }
        state
    }

    /// Gives all actions
    fn get_actions_list(&self, synth: &Synth) -> Vec<Action> {
        let actions = &synth.crafter.actions;
        self.iter().take_while(|m| **m > 0).flat_map(|m| actions.get(*m - 1).map(|m| *m)).collect()
    }

    /// Gives all actions up until the state went invalid
    fn get_final_actions_list(&self, synth: &Synth) -> (State, Vec<Action>) {
        let actions = self.get_actions_list(synth);
        let state = self.calculate_final_state(&synth);
        let (first, _) = actions.split_at(state.step as usize);
        (state, first.iter().map(|m| *m).collect())
    }
}

impl FitnessFunction<CrafterActions, i32> for Synth {
    fn fitness_of(&self, actions: &CrafterActions) -> i32 {
        let state = actions.calculate_final_state(self);
        let violations = state.check_violations();
        let penalties = state.calculate_penalties(10000.0) as i32;
        let mut fitness = if self.solver_vars.solve_for_completion {
            (state.cp_state * self.solver_vars.remainder_cp_fitness_value) + (state.durability_state * self.solver_vars.remainder_dur_fitness_value)
        } else {
            state.quality_state.min(self.recipe.max_quality as i32)
        };
        fitness -= penalties;
        if violations.progress_ok && state.quality_state as f64 >= self.recipe.max_quality as f64 * 1.1 {
            fitness = (fitness as f64 * (1 as f64 + 4 as f64 / state.step as f64)) as i32;
        }

        fitness
    }

    fn average(&self, a: &[i32]) -> i32 {
        a.iter().sum::<i32>() / a.len() as i32
    }

    fn highest_possible_fitness(&self) -> i32 {
        // I believe this helps the solver- worth figuring out math to help this.

        (self.recipe.difficulty + self.recipe.max_quality * 5) as i32

    }

    fn lowest_possible_fitness(&self) -> i32 {
        i32::MIN
    }
}

struct CrafterFitness;

#[wasm_bindgen]
pub struct CraftSimulator {
    pub(crate) generations: u32,
    // extra copy of our synth.
    pub(crate) synth: Synth,
    // oh god this type is so long.
    pub(crate) sim: Simulator<
        GeneticAlgorithm<
            CrafterActions,
            i32,
            Synth,
            MaximizeSelector,
            SinglePointCrossBreeder,
            RandomValueMutator<CrafterActions>,
            ElitistReinserter<CrafterActions, i32, Synth>,
        >,
        GenerationLimit,
    >,
}

impl CraftSimulator {
    pub fn new(synth: Synth) -> Self {
        let number_of_available_actions = synth.crafter.actions.len();
        let number_of_generations = synth.solver_vars.generations;
        let population_size = synth.solver_vars.population;
        let initial_population: Population<CrafterActions> = build_population()
            .with_genome_builder(
                ValueEncodedGenomeBuilder::new(
                50,
                0,                               // define 0 as no operation, end of sequence
                number_of_available_actions + 1, // 1 is our real first ability
            ))
            .of_size(population_size as usize)
            .uniform_at_random();
        let sim = simulate(
            genetic_algorithm()
                .with_evaluation(synth.clone())
                .with_selection(MaximizeSelector::new(0.85, 12))
                .with_crossover(SinglePointCrossBreeder::new())
                .with_mutation(RandomValueMutator::new(0.2, 0, number_of_available_actions))
                .with_reinsertion(ElitistReinserter::new(synth.clone(), false, 0.85))
                .with_initial_population(initial_population)
                .build(),
        )
        .until(GenerationLimit::new(number_of_generations as u64))
        .build();

        Self { generations: 0, synth, sim }
    }

    pub fn next(&mut self) -> SimStep {
        self.generations += 1;
        match self.sim.step() {
            Ok(ok) => match ok {
                SimResult::Intermediate(a) => {
                    eprintln!("{:?}", a.result.best_solution.solution);
                    let genome = &a.result.best_solution.solution.genome;
                    let (state, best_sequence) = genome.get_final_actions_list(&self.synth);
                    log(&format!("gen: {} processing time {}, best fitness {} actions {:?}", self.generations, a.processing_time, a.result.best_solution.solution.fitness, best_sequence));
                    SimStep::Progress {
                        generations_completed: self.generations,
                        max_generations: self.synth.solver_vars.generations as u32,
                        best_sequence,
                        state: state.into(),
                    }
                }
                SimResult::Final(a, b, c, d) => {
                    let genome = &a.result.best_solution.solution.genome;
                    let (state, steps) = genome.get_final_actions_list(&self.synth);
                    SimStep::Complete {
                        best_sequence: steps,
                        execution_log: "blah".to_string(),
                        elapsed_time: None
                    }
                }
            },
            Err(e) => SimStep::Error(e.to_string()),
        }
    }
}

/// State that gets posted to the JS
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatusState {
    quality: i32,
    durability: i32,
    cp: i32,
    progress: i32,
    hq_percent: f64,
    feasible: bool,
    violations: Violations,
    condition: Condition,
    bonus_max_cp: i32
}


impl From<State> for StatusState {
    fn from(state: State) -> Self {
        let violations = state.check_violations();
        Self {
            quality: state.quality_state,
            durability: state.durability_state,
            cp: state.cp_state,
            progress: state.progress_state,
            hq_percent: 0.0, // TODO hq percent calculation
            feasible: violations.is_okay() && violations.progress_ok,
            violations,
            condition: state.condition,
            bonus_max_cp: state.bonus_max_cp
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SimStep {
    #[serde(rename_all = "camelCase")]
    Complete{best_sequence: Vec<Action>, execution_log: String, elapsed_time: Option<i64>},
    #[serde(rename_all = "camelCase")]
    Progress {generations_completed: u32, max_generations: u32, best_sequence: Vec<Action>, state: StatusState},
    Error(String),
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
impl CraftSimulator {
    pub fn new_wasm(synth: &JsValue, max_length: usize, population_size: usize) -> Self {
        console_error_panic_hook::set_once();
        log(&format!("RUST SEES OBJECT {:?} {} {}", synth, max_length, population_size));
        let synth = synth.into_serde().unwrap();
        log(&format!("Loaded synth {:?}", &synth));
        Self::new(synth)
    }

    pub fn next_wasm(&mut self) -> JsValue {
        JsValue::from_serde(&self.next()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Action;
    use crate::simulator::{CraftSimulator, SimStep, CrafterActions, CalcState};
    use crate::xiv_model::{Crafter, Recipe, Synth, SolverVars};
    use serde::Serialize;
    use genevo::genetic::FitnessFunction;

    const TEST_STR:&str  = r#"{"crafter":{"level":90,"craftsmanship":5000,"control":5000,"cp":800,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork2","intensiveSynthesis","prudentSynthesis","delicateSynthesis","focusedSynthesisCombo","focusedTouchCombo","basicTouch","standardTouch","advancedTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","trainedFinesse","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","manipulation","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Alchemist","level":560,"difficulty":2625,"durability":80,"startQuality":0,"maxQuality":4320,"baseLevel":90,"progressDivider":130,"progressModifier":90,"qualityDivider":115,"qualityModifier":80,"suggestedControl":2635,"suggestedCraftsmanship":2805,"name":"Sharlayan Fishcake Ingredient"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
    const SMOL_ABILITY: &str = r#"{"crafter":{"level":9,"craftsmanship":100,"control":100,"cp":180,"actions":["basicSynth","basicTouch","mastersMend"]},"recipe":{"baseLevel":10,"difficulty":45,"durability":60,"level":10,"maxQuality":250,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":29,"suggestedCraftsmanship":59,"name":"Heat Vent Component","cls":"Culinarian","startQuality":0},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
    #[test]
    fn test_json_synth() {
        let json_str = TEST_STR;
        let synth : Synth = serde_json::from_str(json_str).unwrap();
        let mut sim = CraftSimulator::new(synth);
        let value = sim.next();
        let value = sim.next();
        let value = sim.next();
        let step = sim.next();

        match step {
            SimStep::Complete { .. } => {
                assert!(false);
            }
            SimStep::Progress { state, .. } => {

            }
            SimStep::Error(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn valid_crafter_actions() {
        let valid_rotation : CrafterActions = vec![1, 1, 2, 2, 0, 1, 2, 3, 1];
        let synth : Synth = serde_json::from_str(&SMOL_ABILITY).unwrap();
        let expected_actions = vec![Action::BasicSynth, Action::BasicSynth, Action::BasicTouch, Action::BasicTouch];
        let actions = valid_rotation.get_actions_list(&synth);
        assert_eq!(actions, expected_actions);

        let (state, action) = valid_rotation.get_final_actions_list(&synth);
        assert_eq!(action, expected_actions);
        assert_ne!(state.step, 0);
    }

    #[test]
    fn empty_action_list() {
        let numbers : CrafterActions = vec![0,0,25,26,7,3,10,1];
        let synth : Synth = serde_json::from_str(TEST_STR).unwrap();
        assert_eq!(numbers.get_actions_list(&synth), vec![]);
        let fitness = synth.fitness_of(&numbers);
        assert!(fitness < 0);
    }

    #[test]
    fn test_real_actions() {
        let crafter_data = r#"{"crafter":{"level":82,"craftsmanship":2606,"control":2457,"cp":507,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork","intensiveSynthesis","delicateSynthesis","focusedSynthesisCombo","focusedTouchCombo","basicTouch","standardTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Alchemist","level":430,"difficulty":1780,"durability":80,"startQuality":0,"maxQuality":4600,"baseLevel":80,"progressDivider":110,"progressModifier":100,"qualityDivider":90,"qualityModifier":100,"suggestedControl":1733,"suggestedCraftsmanship":1866,"name":"Tincture of Strength"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
        let synth : Synth = serde_json::from_str(crafter_data).unwrap();
        let mut sim = CraftSimulator::new(synth);
        while let SimStep::Progress { .. } = sim.next() {

        }

    }

    #[test]
    fn lvl50_cul_synth() {
        let synth : Synth = serde_json::from_str(r#"{"crafter":{"level":51,"craftsmanship":117,"control":158,"cp":180,"actions":["basicSynth2","basicTouch","standardTouch","byregotsBlessing","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Culinarian","level":40,"difficulty":138,"durability":60,"startQuality":0,"maxQuality":3500,"baseLevel":40,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":68,"suggestedCraftsmanship":136,"name":"Grade 4 Skybuilders' Sesame Cookie"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#).unwrap();
        let mut sim = CraftSimulator::new(synth);
        let next = sim.next();
        match next {
            SimStep::Complete { .. } => {
                assert!(false);
            }
            SimStep::Progress { best_sequence, .. } => {
                assert_ne!(best_sequence, vec![]);
            }
            SimStep::Error(_) => {
                assert!(false);
            }
        }
    }

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
            stars: None,
        };
        let crafter = Crafter {
            //cls: 10,
            craftsmanship: 20,
            control: 20,
            craft_points: 10,
            level: 10,
            specialist: false,
            actions: vec![Action::BasicSynth, Action::StandardTouch],
        };
        let synth = Synth {
            crafter,
            recipe,
            max_trick_uses: 10,
            reliability_percent: 1,
            max_length: 50,
            solver_vars: SolverVars {
                max_stagnation_counter: 0,
                population: 5000,
                generations: 750,
                ..Default::default()
            },
        };

        let mut sim = CraftSimulator::new(synth);
        let sim_result = sim.next();
        match sim_result {
            SimStep::Complete { .. } => {assert!(false)}
            SimStep::Progress { generations_completed, max_generations, best_sequence, state } => {
                assert_ne!(best_sequence.len(), 0);
                //assert_ne!(state.step, 0);
            }
            SimStep::Error(_) => {assert!(false)}
        }


    }
}
