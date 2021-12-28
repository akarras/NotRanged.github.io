use crate::actions::Action;
use crate::xiv_model::{Condition, SimulationCondition, State, Synth, Violations};
use genevo::ga::genetic_algorithm;
use genevo::operator::prelude::*;
use genevo::population::ValueEncodedGenomeBuilder;
use genevo::prelude::*;
use genevo::prelude::{simulate, FitnessFunction, GenerationLimit, Simulation, SimulationBuilder};
use genevo::simulation::simulator::Simulator;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

// genotype, usize where index matches available action
pub type CrafterActions = Vec<usize>;

trait CalcState {
    fn calculate_final_state<'a>(&self, synth: &'a Synth, log: &mut Option<String>) -> State<'a>;

    fn get_actions_list(&self, synth: &Synth) -> Vec<Action>;

    fn get_final_actions_list<'a>(
        &self,
        synth: &'a Synth,
        log: &mut Option<String>,
    ) -> (State<'a>, Vec<Action>);
}

impl CalcState for CrafterActions {
    fn calculate_final_state<'a>(&self, synth: &'a Synth, log: &mut Option<String>) -> State<'a> {
        let mut state: State = synth.into();
        let mut condition = SimulationCondition::new_sim_condition();
        if let Some(log) = log {
            let _ = writeln!(log, "{}", state);
        }
        for action in self
            .iter()
            .take_while(|m| **m > 0)
            .flat_map(|m| synth.crafter.actions.get(*m - 1).copied())
        {
            let tmp_state = state.add_action(action, &mut condition);
            if let Some(log) = log {
                let _ = writeln!(log, "{}", tmp_state);
            }
            if tmp_state.progress_state >= synth.recipe.difficulty as i32 {
                return tmp_state;
            }
            if tmp_state.durability_state <= 0 {
                return tmp_state; // bad durability, no point proceeding
            }
            if tmp_state.cp_state <= 0 {
                return state;
            }
            state = tmp_state;
        }
        state
    }

    /// Gives all actions
    fn get_actions_list(&self, synth: &Synth) -> Vec<Action> {
        let actions = &synth.crafter.actions;
        self.iter()
            .take_while(|m| **m > 0)
            .flat_map(|m| actions.get(*m - 1).copied())
            .collect()
    }

    /// Gives all actions up until the state went invalid
    fn get_final_actions_list<'a>(
        &self,
        synth: &'a Synth,
        log: &mut Option<String>,
    ) -> (State<'a>, Vec<Action>) {
        let actions = self.get_actions_list(synth);
        let state = self.calculate_final_state(synth, log);
        let (first, _) = actions.split_at(state.step as usize);
        (state, first.to_vec())
    }
}

impl FitnessFunction<CrafterActions, i32> for Synth {
    fn fitness_of(&self, actions: &CrafterActions) -> i32 {
        let state = actions.calculate_final_state(self, &mut None);
        let violations = state.check_violations();
        let penalties = state.calculate_penalties(10000.0) as i32;
        let mut fitness = if self.solver_vars.solve_for_completion {
            (state.cp_state * self.solver_vars.remainder_cp_fitness_value)
                + (state.durability_state * self.solver_vars.remainder_dur_fitness_value)
        } else {
            state.quality_state.min(self.recipe.max_quality as i32)
        };
        fitness -= penalties;
        let safety_margin_factor = 1.0 + self.recipe.safety_margin as f64 * 0.01;
        if violations.progress_ok
            && state.quality_state as f64 >= self.recipe.max_quality as f64 * safety_margin_factor
        {
            fitness = (fitness as f64 * (1.0 + 4.0 / state.step as f64)) as i32;
        }

        fitness
    }

    fn average(&self, a: &[i32]) -> i32 {
        (a.iter().map(|m| *m as i64).sum::<i64>() / a.len() as i64) as i32
    }

    fn highest_possible_fitness(&self) -> i32 {
        // I believe this helps the solver- worth figuring out math to help this.

        (self.recipe.difficulty + self.recipe.max_quality * 5) as i32
    }

    fn lowest_possible_fitness(&self) -> i32 {
        i32::MIN
    }
}

type GeneticSimulator = Simulator<
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
>;

#[wasm_bindgen]
pub struct CraftSimulator {
    pub(crate) generations: u32,
    // extra copy of our synth.
    pub(crate) synth: Synth,
    // oh god this type is so long.
    pub(crate) sim: GeneticSimulator,
}

impl CraftSimulator {
    pub fn new(synth: Synth) -> Self {
        let number_of_available_actions = synth.crafter.actions.len();
        let number_of_generations = synth.solver_vars.generations;

        #[cfg(feature = "wasm-thread")]
            let population_size = {
                log(&format!("USING {} cores", rayon::current_num_threads()));
                synth.solver_vars.population / rayon::current_num_threads() as i32
            };
        #[cfg(not(feature = "wasm-thread"))]
        let population_size = synth.solver_vars.population;
        let initial_population: Population<CrafterActions> = build_population()
            .with_genome_builder(ValueEncodedGenomeBuilder::new(
                50,
                0,                               // define 0 as no operation, end of sequence
                number_of_available_actions + 1, // 1 is our real first ability
            ))
            .of_size(population_size as usize)
            .uniform_at_random();
        log(&format!("population_size: {}", population_size));
        let sim = simulate(
            genetic_algorithm()
                .with_evaluation(synth.clone())
                .with_selection(MaximizeSelector::new(0.85, 18))
                .with_crossover(SinglePointCrossBreeder::new())
                .with_mutation(RandomValueMutator::new(
                    0.2,
                    0,
                    number_of_available_actions + 1,
                ))
                .with_reinsertion(ElitistReinserter::new(synth.clone(), false, 0.85))
                .with_initial_population(initial_population)
                .build(),
        )
        .until(GenerationLimit::new(number_of_generations as u64))
        .build();

        Self {
            generations: 0,
            synth,
            sim,
        }
    }

    pub fn next_generation(&mut self) -> SimStep {
        self.generations += 1;
        match self.sim.step() {
            Ok(ok) => match ok {
                SimResult::Intermediate(a) => {
                    let genome = &a.result.best_solution.solution.genome;
                    let mut work_log = Some(String::new());
                    let (state, best_sequence) =
                        genome.get_final_actions_list(&self.synth, &mut work_log);
                    #[cfg(target_arch = "wasm32")]
                    log(&format!(
                        "gen: {} {}, best fitness {} actions {:?}\n worklog:\n{}",
                        self.generations,
                        a.processing_time,
                        a.result.best_solution.solution.fitness,
                        best_sequence,
                        work_log.unwrap()
                    ));
                    SimStep::Progress {
                        generations_completed: self.generations,
                        max_generations: self.synth.solver_vars.generations as u32,
                        best_sequence,
                        state: state.into(),
                    }
                }
                SimResult::Final(a, b, c, d) => {
                    let genome = &a.result.best_solution.solution.genome;
                    let mut log = Some("Final State Log\n".to_string());
                    let (state, steps) = genome.get_final_actions_list(&self.synth, &mut log);
                    let mut log = log.unwrap();
                    let _ = write!(
                        log,
                        "\nFinal State: \n{:#?}\nDuration {}\n Stop Reason: {}",
                        state, c, d
                    );
                    SimStep::Success {
                        best_sequence: steps,
                        execution_log: log,
                        elapsed_time: Some(b.duration().num_seconds()),
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
    bonus_max_cp: i32,
}

impl From<State<'_>> for StatusState {
    fn from(state: State<'_>) -> Self {
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
            bonus_max_cp: state.bonus_max_cp,
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SimStep {
    #[serde(rename_all = "camelCase")]
    Success {
        best_sequence: Vec<Action>,
        execution_log: String,
        elapsed_time: Option<i64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        generations_completed: u32,
        max_generations: u32,
        best_sequence: Vec<Action>,
        state: StatusState,
    },
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
    pub fn new_wasm(synth: &JsValue) -> Self {
        console_error_panic_hook::set_once();
        log(&format!("RUST SEES OBJECT {:?}", synth));
        let synth = synth.into_serde().unwrap();
        log(&format!("Loaded synth {:?}", &synth));
        Self::new(synth)
    }

    pub fn next_wasm(&mut self) -> JsValue {
        JsValue::from_serde(&self.next_generation()).unwrap()
    }

    pub fn pause_wasm(&mut self) -> JsValue {
        let mut value = self.next_generation();
        if let SimStep::Progress { generations_completed, max_generations, best_sequence, state } = value {
            value = SimStep::Success {
                best_sequence,
                execution_log: "".to_string(),
                elapsed_time: None
            };
        }

        JsValue::from_serde(&value).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Action;
    use crate::simulator::{CalcState, CraftSimulator, CrafterActions, SimStep};
    use crate::xiv_model::{Crafter, Recipe, SolverVars, Synth};
    use genevo::genetic::FitnessFunction;

    const TEST_STR: &str = r#"{"crafter":{"level":90,"craftsmanship":5000,"control":5000,"cp":800,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork2","intensiveSynthesis","prudentSynthesis","delicateSynthesis","focusedSynthesisCombo","focusedTouchCombo","basicTouch","standardTouch","advancedTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","trainedFinesse","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","manipulation","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Alchemist","level":560,"difficulty":2625,"durability":80,"startQuality":0,"maxQuality":4320,"baseLevel":90,"progressDivider":130,"progressModifier":90,"qualityDivider":115,"qualityModifier":80,"suggestedControl":2635,"suggestedCraftsmanship":2805,"name":"Sharlayan Fishcake Ingredient"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
    const SMOL_ABILITY: &str = r#"{"crafter":{"level":9,"craftsmanship":100,"control":100,"cp":180,"actions":["basicSynth","basicTouch","mastersMend"]},"recipe":{"baseLevel":10,"difficulty":45,"durability":60,"level":10,"maxQuality":250,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":29,"suggestedCraftsmanship":59,"name":"Heat Vent Component","cls":"Culinarian","startQuality":0},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;

    #[test]
    fn valid_crafter_actions() {
        let valid_rotation: CrafterActions = vec![1, 1, 2, 2, 0, 1, 2, 3, 1];
        let synth: Synth = serde_json::from_str(&SMOL_ABILITY).unwrap();
        let expected_actions = vec![
            Action::BasicSynth,
            Action::BasicSynth,
            Action::BasicTouch,
            Action::BasicTouch,
        ];
        let actions = valid_rotation.get_actions_list(&synth);
        assert_eq!(actions, expected_actions);

        let (state, action) = valid_rotation.get_final_actions_list(&synth, &mut None);
        assert_eq!(action, expected_actions);
        assert_ne!(state.step, 0);
    }

    #[test]
    fn empty_action_list() {
        let numbers: CrafterActions = vec![0, 0, 25, 26, 7, 3, 10, 1];
        let synth: Synth = serde_json::from_str(TEST_STR).unwrap();
        assert_eq!(numbers.get_actions_list(&synth), vec![]);
        let fitness = synth.fitness_of(&numbers);
        assert!(fitness < 0);
    }

    #[test]
    fn test_real_actions() {
        let crafter_data = r#"{"crafter":{"level":82,"craftsmanship":2606,"control":2457,"cp":507,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork","intensiveSynthesis","delicateSynthesis","focusedSynthesisCombo","focusedTouchCombo","basicTouch","standardTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Alchemist","level":430,"difficulty":1780,"durability":80,"startQuality":0,"maxQuality":4600,"baseLevel":80,"progressDivider":110,"progressModifier":100,"qualityDivider":90,"qualityModifier":100,"suggestedControl":1733,"suggestedCraftsmanship":1866,"name":"Tincture of Strength"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
        let synth: Synth = serde_json::from_str(crafter_data).unwrap();
        let mut sim = CraftSimulator::new(synth);
        while let SimStep::Progress { .. } = sim.next_generation() {}
    }

    #[test]
    fn lvl50_cul_synth() {
        let synth : Synth = serde_json::from_str(r#"{"crafter":{"level":51,"craftsmanship":117,"control":158,"cp":180,"actions":["basicSynth2","basicTouch","standardTouch","byregotsBlessing","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Culinarian","level":40,"difficulty":138,"durability":60,"startQuality":0,"maxQuality":3500,"baseLevel":40,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":68,"suggestedCraftsmanship":136,"name":"Grade 4 Skybuilders' Sesame Cookie"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#).unwrap();
        let mut sim = CraftSimulator::new(synth);
        let next = sim.next_generation();
        match next {
            SimStep::Success { .. } => {
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
            safety_margin: 0,
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
        let sim_result = sim.next_generation();
        match sim_result {
            SimStep::Success { .. } => {
                assert!(false)
            }
            SimStep::Progress { best_sequence, .. } => {
                assert_ne!(best_sequence.len(), 0);
                //assert_ne!(state.step, 0);
            }
            SimStep::Error(_) => {
                assert!(false)
            }
        }
    }
}
