use crate::actions::{Action, ActionType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use crate::level_table;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct Crafter {
    cls: u32,
    craftsmanship: u32,
    control: u32,
    craft_points: u32,
    level: u32,
    specialist: u32,
    actions: Vec<Action>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct Recipe {
    base_level: u32,
    level: u32,
    difficulty: u32,
    durability: u32,
    start_quality: u32,
    max_quality: u32,
    suggested_craftsmanship: u32,
    suggested_control: u32,
    progress_divider: f64,
    progress_modifier: Option<u32>,
    quality_divider: f64,
    quality_modifier: Option<u32>,
    stars: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct SolverVars {
    solve_for_completion: bool,
    #[serde(rename = "remainderCPFitnessValue")]
    remainder_cp_fitness_value: bool,
    remainder_dur_fitness_value: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct Synth {
    crafter: Crafter,
    recipe: Recipe,
    max_trick_uses: i32,
    reliability_index: u32,
    max_length: u32,
    solver_vars: SolverVars,
}

impl Synth {
    fn calculate_base_progress_increase(&self, eff_crafter_level: u32, craftsmanship: u32) -> u32 {
        let base_value: f64 = (craftsmanship as f64 * 10.0) / self.recipe.progress_divider + 2.0;
        if eff_crafter_level <= self.recipe.level {
            (base_value * (self.recipe.progress_modifier.unwrap_or(100) as f64) / 100.0) as u32
        } else {
            base_value as u32
        }
    }

    fn calculate_base_quality_increase(&self, eff_crafter_level: u32, control: u32) -> u32 {
        let base_value: f64 = (control as f64 * 10.0) / self.recipe.quality_divider + 35.0;
        if eff_crafter_level <= self.recipe.base_level {
            (base_value * (self.recipe.quality_modifier.unwrap_or(100) as f64) / 100.0) as u32
        } else {
            base_value as u32
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Condition {
    Good,
    Normal,
    Excellent,
    Poor,
}

impl Default for Condition {
    fn default() -> Self {
        Condition::Good
    }
}

impl Condition {
    fn check_good_or_excellent(&self) -> bool {
        match self {
            Condition::Good => true,
            Condition::Excellent => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct EffectTracker {
    count_ups: u32,
    count_downs: u32,
    indefinites: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Effects {
    count_downs: BTreeMap<Action, i32>,
    count_ups: BTreeMap<Action, i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct State {
    synth: Synth,
    step: u32,
    last_step: u32,
    action: Option<Action>, // Action leading to this state
    durability_state: i32,
    cp_state: i32,
    bonus_max_cp: i32,
    quality_state: i32,
    progress_state: i32,
    wasted_actions: f64,
    trick_uses: i32,
    name_of_element_uses: i32,
    reliability: i32,
    effects: Effects,
    condition: Condition,

    // Advancedtouch combo stuff
    touch_combo_step: i32,

    // Internal state variables set after each step.
    iq_cnt: i32,
    control: i32,
    quality_gain: i32,
    progress_gain: bool,
    b_quality_gain: bool, // Rustversion: for some reason these are almost the same name?
    success: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Violations {
    progress_ok: bool,
    cp_ok: bool,
    durability_ok: bool,
    trick_ok: bool,
    reliability_ok: bool,
}

impl State {
    fn check_violations(&self) -> Violations {
        let progress_ok = self.progress_state >= self.synth.recipe.difficulty as i32;
        let cp_ok = self.cp_state >= 0;
        let durability_ok = if self.durability_state >= -5
            && self.progress_state >= self.synth.recipe.difficulty as i32
        {
            if self.action.unwrap().details().durability_cost == 10 || self.durability_state >= 0 {
                true
            } else {
                false
            }
        } else {
            false
        };

        let trick_ok = self.trick_uses <= self.synth.max_trick_uses;
        let reliability_ok = self.reliability > self.synth.reliability_index as i32;
        Violations {
            progress_ok,
            cp_ok,
            durability_ok,
            trick_ok,
            reliability_ok,
        }
    }
}

impl From<&Synth> for State {
    fn from(synth: &Synth) -> Self {
        State {
            synth: synth.clone(),
            step: 0,
            last_step: 0,
            action: None,
            effects: Effects {
                count_ups: [(Action::InnerQuiet, 1)].into_iter().collect(),
                ..Default::default()
            },
            reliability: 1,
            condition: Condition::Normal,
            ..Default::default()
        }
    }
}

fn prob_good_for_synth(synth: &Synth) -> f64 {
    let recipe_level = synth.recipe.level;
    let quality_assurance = synth.crafter.level >= 63;
    if recipe_level >= 300 {
        // 70+
        match quality_assurance {
            true => 0.11,
            false => 0.10,
        }
    } else if recipe_level >= 276 {
        // 65+
        match quality_assurance {
            true => 0.17,
            false => 0.15,
        }
    } else if recipe_level >= 255 {
        // 61+
        match quality_assurance {
            true => 0.22,
            false => 0.20,
        }
    } else if recipe_level >= 150 {
        // 55+
        match quality_assurance {
            true => 0.17,
            false => 0.15,
        }
    } else {
        match quality_assurance {
            true => 0.27,
            false => 0.25,
        }
    }
}

fn prob_excellent_for_synth(synth: &Synth) -> f64 {
    let recipe_level = synth.recipe.level;
    if recipe_level >= 300 {
        // 70*+
        0.01
    } else if recipe_level >= 255 {
        // 65+
        0.02
    } else if recipe_level >= 150 {
        // 60+
        0.01
    } else {
        0.02
    }
}

fn get_effective_crafter_level(synth: &Synth) -> u32 {
    let eff_crafter_level = synth.crafter.level;
    level_table::level_table_lookup(eff_crafter_level)
}

struct ModifierResult {
    craftsmanship: u32,
    control: u32,
    eff_crafter_level: u32,
    eff_recipe_level: u32,
    level_difference: u32,
    success_probability: f64,
    quality_increase_multiplier: f64,
    progress_gain: f64,
    quality_gain: u32,
    durability_cost: f64,
    cp_cost: i32,
}

/// I could just do the functions that the JS uses, but I did it this way and I'm too lazy to change it now.
enum SimulationCondition {
    MonteCarlo { ignore_condition_req: bool },
}

impl SimulationCondition {
    fn check_good_or_excellent(&self, state: &State) -> bool {
        match self {
            SimulationCondition::MonteCarlo {
                ignore_condition_req,
            } => {
                if *ignore_condition_req {
                    true
                } else {
                    state.condition.check_good_or_excellent()
                }
            }
        }
    }

    fn p_good_or_excellent(&self, state: &State) -> f64 {
        match self {
            SimulationCondition::MonteCarlo { .. } => 1.0,
        }
    }
}

fn apply_modifiers(state: &mut State, action: Action, condition: &SimulationCondition) {
    let craftsmanship = state.synth.crafter.craftsmanship;
    let mut control = state.synth.crafter.control;
    let mut cp_cost = action.details().cp_cost;

    // Effects modifying level difference
    let eff_crafter_level = get_effective_crafter_level(&state.synth);
    let eff_recipe_level = state.synth.recipe.level;
    let level_difference = eff_crafter_level - eff_recipe_level;
    let original_level_difference = eff_crafter_level - eff_recipe_level;
    let pure_level_difference = state.synth.crafter.level - state.synth.recipe.base_level;
    let recipe_level = eff_recipe_level;
    let stars = state.synth.recipe.stars;

    // Effects modifying probability
    let action_details = action.details();
    let mut success_probability = action_details.success_probability;
    if action.eq(&Action::FocusedSynthesis) || action.eq(&Action::FocusedTouch) {
        if let Some(sa) = &state.action {
            if sa.eq(&Action::Observe) {
                success_probability = 1.0;
            }
        }
    }

    success_probability = success_probability.min(1.0);

    // Advanced Touch Combo
    if action.eq(&Action::AdvancedTouch) {
        if let Some(sa) = &state.action {
            if *sa == Action::StandardTouch && state.touch_combo_step == 1 {
                state.touch_combo_step = 0;
                cp_cost = 18;
            }
        }
    }
    // Add combo bonus following Basic Touch
    if action.eq(&Action::StandardTouch) {
        if let Some(sa) = &state.action {
            if *sa == Action::BasicTouch {
                cp_cost = 18;
                state.wasted_actions -= 0.05;
                state.touch_combo_step = 1;
            }
            if *sa == Action::StandardTouch {
                state.wasted_actions += 0.1;
            }
        }
    }

    // Penalize use of WasteNot during solveforcompletion runs

    if action == Action::WasteNot
        || action == Action::WasteNot2 && state.synth.solver_vars.solve_for_completion
    {
        state.wasted_actions += 50.0;
    }

    // Effects modifying progress increase multiplier
    let mut progress_increase_multiplier = 1.0;

    if (action_details.progress_increase_multiplier > 0.0)
        && (state
        .effects
        .count_downs
        .contains_key(&Action::MuscleMemory))
    {
        progress_increase_multiplier += 1.0;
        //delete state.effects.count_downs[AllActions.muscleMemory.shortName];
    }

    if state.effects.count_downs.contains_key(&Action::Veneration) {
        progress_increase_multiplier += 0.5;
    }

    if action.eq(&Action::MuscleMemory) {
        if state.step != 1 {
            state.wasted_actions += 1.0;
            progress_increase_multiplier = 0.0;
            cp_cost = 0;
        }
    }
    // TODO do we need to be applying the durability cost from waste not to this?
    if state.durability_state < action_details.durability_cost {
        if action == Action::Groundwork || action == Action::Groundwork2 {
            progress_increase_multiplier *= 0.5;
        }
    }

    // Effects modifying quality increase multiplier
    let mut quality_increase_multiplier = 1.0;
    let mut quality_increase_multiplier_iq = 1.0; // This is calculated seperately because it's multiplicative instead of additive! See: how teamcrafting does it

    if state
        .effects
        .count_downs
        .contains_key(&Action::GreatStrides)
        && quality_increase_multiplier > 0.0
    {
        quality_increase_multiplier += 1.0;
    }

    if state.effects.count_downs.contains_key(&Action::Innovation) {
        quality_increase_multiplier += 0.5;
    }

    if let Some(inner_quiet_value) = state.effects.count_ups.get(&Action::InnerQuiet) {
        quality_increase_multiplier_iq += 0.1 * (inner_quiet_value + 1) as f64
        // +1 because buffs start incrementing from 0
    }

    // We can only use Byregot actions when we have at least 1 stacks of inner quiet
    if action == Action::ByregotsBlessing {
        let num_inner_quiets = *state
            .effects
            .count_ups
            .get(&Action::InnerQuiet)
            .unwrap_or(&0);
        if state
            .effects
            .count_ups
            .get(&Action::InnerQuiet)
            .map(|c| c >= &1)
            .unwrap_or_default()
        {
            quality_increase_multiplier *= 1.0 + (0.2 * (num_inner_quiets + 1) as f64).min(3.0);
        } else {
            quality_increase_multiplier = 0.0;
        }
    }

    // Calculate base and modified progress gain
    let progress_gain = state
        .synth
        .calculate_base_progress_increase(eff_crafter_level, craftsmanship);
    let mut progress_gain = progress_gain as f64
        * action_details.progress_increase_multiplier
        * progress_increase_multiplier;

    // Calculate base and modified quality gain
    let mut quality_gain = state
        .synth
        .calculate_base_quality_increase(eff_crafter_level, control);
    // conversion back to u32 from f64 is equivalent to .floor().
    quality_gain = (quality_gain as f64
        * action_details.quality_increase_multiplier
        * quality_increase_multiplier
        * quality_increase_multiplier_iq) as u32;

    // Trained finesse
    if action.eq(&Action::TrainedFinesse) {
        // Not at 10 stacks of IQ -> wasted action
        if *state
            .effects
            .count_ups
            .get(&Action::InnerQuiet)
            .unwrap_or(&0)
            != 9
        {
            state.wasted_actions += 1.0;
            quality_gain = 0;
        }
    }

    // Effects modifying durability cost
    let mut durability_cost = action_details.durability_cost as f64;
    if state.effects.count_downs.contains_key(&Action::WasteNot)
        || state.effects.count_downs.contains_key(&Action::WasteNot2)
    {
        if action.eq(&Action::PrudentTouch) {
            quality_gain = 0;
            state.wasted_actions += 1.0;
        } else if action.eq(&Action::PrudentSynthesis) {
            progress_gain = 0.0;
            state.wasted_actions += 1.0;
        } else {
            durability_cost *= 0.5;
        }
    }

    // Effects modifying quality gain directly
    if action.eq(&Action::TrainedEye) {
        if state.step == 1 && pure_level_difference >= 10 && !state.synth.recipe.stars {
            quality_gain = state.synth.recipe.max_quality;
        } else {
            state.wasted_actions += 1.0;
            quality_gain = 0;
            cp_cost = 0;
        }
    }

    // We can only use Precise Touch when state material condition is Good or Excellent. Default is true for probabilistic method.
    if action.eq(&Action::PreciseTouch) {
        if condition.check_good_or_excellent(state) {
            quality_gain *= condition.p_good_or_excellent(state) as u32;
        } else {
            state.wasted_actions += 1.0;
            quality_gain = 0;
            cp_cost = 0;
        }
    }

    if action.eq(&Action::Reflect) {
        if state.step != 1 {
            state.wasted_actions += 1.0;
            control = 0;
            quality_gain = 0;
            cp_cost = 0;
        }
    }

    ModifierResult {
        craftsmanship,
        control,
        eff_crafter_level,
        eff_recipe_level,
        level_difference,
        success_probability,
        quality_increase_multiplier,
        progress_gain,
        quality_gain,
        durability_cost,
        cp_cost,
    };
}

fn use_conditional_action(state: &mut State, condition: &SimulationCondition) -> bool {
    if state.cp_state > 0 && condition.check_good_or_excellent(state) {
        state.trick_uses += 1;
        return true;
    } else {
        state.wasted_actions += 1.0;
        return false;
    }
}

fn apply_special_action_effects(
    state: &mut State,
    action: Action,
    condition: &SimulationCondition,
) {
    // STEP_02
    // Effect management
    //==================================
    // Special Effect Actions
    if action == Action::MastersMend {
        state.durability_state += 30;
        if state.synth.solver_vars.solve_for_completion {
            state.wasted_actions += 50.0; // Bad code, but it works. We don't want dur increase in solveforcompletion.
        }
    }

    if state
        .effects
        .count_downs
        .contains_key(&Action::Manipulation)
        && state.durability_state > 0
        && action != Action::Manipulation
    {
        state.durability_state += 5;
        if state.synth.solver_vars.solve_for_completion {
            state.wasted_actions += 50.0; // Bad code, but it works. We don't want dur increase in solveforcompletion.
        }
    }

    if action == Action::ByregotsBlessing {
        if state.effects.count_ups.contains_key(&Action::InnerQuiet) {
            state.effects.count_ups.remove(&Action::InnerQuiet);
        } else {
            state.wasted_actions += 1.0;
        }
    }

    if action == Action::Reflect {
        if state.step == 1 {
            if let Some(mut count) = state.effects.count_ups.get_mut(&Action::InnerQuiet) {
                *count += 1;
            } else {
                state.effects.count_ups.insert(Action::InnerQuiet, 0); // what does this even get inserted as?
            }
        } else {
            state.wasted_actions += 1.0;
        }
    }
    let action_details = action.details();
    if action_details.quality_increase_multiplier > 0.0
        && state
        .effects
        .count_downs
        .contains_key(&Action::GreatStrides)
    {
        state.effects.count_downs.remove(&Action::GreatStrides);
    }

    // Manage effects with conditional requirements
    if action_details.on_excellent || action_details.on_good {
        if use_conditional_action(state, condition) {
            if action == Action::TricksOfTheTrade {
                state.cp_state += (20.0 * condition.p_good_or_excellent(state)) as i32;
            }
        }
    }

    if action == Action::Veneration && state.effects.count_downs.contains_key(&Action::Veneration) {
        state.wasted_actions += 1.0
    }
    if action == Action::Innovation && state.effects.count_downs.contains_key(&Action::Innovation) {
        state.wasted_actions += 1.0
    }
}


fn update_effects_counters(state: &mut State, action: Action, condition: &SimulationCondition, successProbability: f64) {
    // STEP_03
    // Countdown / Countup Management
    //===============================
    // Decrement countdowns
    let mut remove_values = vec![];
    let action_details = action.details();
    for (action, count) in &mut state.effects.count_downs {
        *count -= 1;
        if *count <= 0 {
            remove_values.push(*action);
        }
    }
    for value in remove_values {
        state.effects.count_downs.remove_entry(&value);
    }


    if state.effects.count_ups.contains_key(&Action::InnerQuiet) {
        // Increment inner quiet countups that have conditional requirements
        if action == Action::PreparatoryTouch {
            if let Some(quiet) = state.effects.count_ups.get_mut(&Action::InnerQuiet) {
                *quiet += 2;
            }
        }
        // Increment inner quiet countups that have conditional requirements
        else if action == Action::PreciseTouch && condition.check_good_or_excellent(state) {
            let quiet_increment = (2.0 * successProbability * condition.p_good_or_excellent(state)) as i32;
            if let Some(quiet) = state.effects.count_ups.get_mut(&Action::InnerQuiet) {
                *quiet += quiet_increment;
            }
        }
        // Increment all other inner quiet count ups
        else if action.details().quality_increase_multiplier > 0.0 && action != Action::Reflect && action != Action::TrainedFinesse {
            if let Some(quiet) = state.effects.count_ups.get_mut(&Action::InnerQuiet) {
                *quiet += (1.0 * successProbability) as i32;
            }
        }

        // Cap inner quiet stacks at 9 (10)
        if let Some(quiet) = state.effects.count_ups.get_mut(&Action::InnerQuiet) {
            *quiet = (*quiet).min(9);
        }
    }

    // Initialize new effects after countdowns are managed to reset them properly
    if action_details.action_type == ActionType::CountUp {
        state.effects.count_ups.insert(action, 0);
    }

    if action_details.action_type == ActionType::Immediate {
        /* TODO is this action even a thing?
        if isActionEq(action, AllActions.initialPreparations) {
            if s.step == 1 {
                s.effects.indefinites[action.shortName] = true;
            }
            else {
                s.wastedActions += 1;
            }
        }
        else {
            s.effects.indefinites[action.shortName] = true;
        }*/
    }

    if let ActionType::Countdown{ active_turns } = action_details.action_type  {
        /* TODO AGAIN, what??
        if (action.shortName.indexOf('nameOf') >= 0) {
            if (s.nameOfElementUses == 0) {
                s.effects.countDowns[action.shortName] = action.activeTurns;
                s.nameOfElementUses += 1;
            }
            else {
                s.wastedActions += 1;
            }
        }*/
        if action == Action::MuscleMemory && state.step != 1 {
            state.wasted_actions += 1.0;
        }
        else {
            state.effects.count_downs.insert(action, active_turns);
            //s.effects.countDowns[action.shortName] = action.activeTurns;
        }
    }
}

fn update_state(state: &mut State, action: Action, progress_gain: i32, quality_gain: i32, durability_cost: i32, cp_cost: i32, condition: &SimulationCondition, success_probability: f64) {
    // State tracking
    state.progress_state += progress_gain;
    state.quality_state += quality_gain;
    state.durability_state -= durability_cost;
    state.cp_state -= cp_cost;
    state.last_step += 1;
    apply_special_action_effects(state, action, condition);
    update_effects_counters(state, action, condition, success_probability);

    // Sanity checks for state variables
    /* Removing this for solveForCompletion, hopefully it doesn't cause issues! :)
    if ((s.durabilityState >= -5) && (s.progressState >= s.synth.recipe.difficulty)) {
        //s.durabilityState = 0;
    }
    */
    state.durability_state = state.durability_state.min(state.synth.recipe.durability as i32);
    state.cp_state = state.cp_state.min(state.synth.crafter.craft_points as i32 + state.bonus_max_cp);
}

/*fn sim_synth(individual: String, start_state: State, assume_success: bool, verbose: bool, debug: bool, log_output: Option<String>) -> State {

    let logger = Logger(logOutput);

    // Clone startState to keep startState immutable
    let state = start_state.clone();

    // Conditions
    let pGood = prob_good_for_synth(&state.synth);
    let pExcellent = prob_excellent_for_synth(&state.synth);
    let ignoreConditionReq = !state.synth.useConditions;

    // Step 1 is always normal
    let mut ppGood = 0;
    let mut ppExcellent = 0;
    let mut ppPoor = 0;
    let mut ppNormal = 1 - (ppGood + ppExcellent + ppPoor);



    // Check for null or empty individuals
    if individual.empty() {
        return State::from(&state.synth)
    }

    if debug {
      //logger.log('%-2s %30s %-5s %-5s %-8s %-8s %-5s %-8s %-8s %-5s %-5s %-5s', '#', 'Action', 'DUR', 'CP', 'EQUA', 'EPRG', 'IQ', 'CTL', 'QINC', 'BPRG', 'BQUA', 'WAC');
      //logger.log('%2d %30s %5.0f %5.0f %8.1f %8.1f %5.1f %8.1f %8.1f %5.0f %5.0f %5.0f', s.step, '', s.durabilityState, s.cpState, s.qualityState, s.progressState, 0, s.synth.crafter.control, 0, 0, 0, 0);
    }
    else if verbose {
      // logger.log('%-2s %30s %-5s %-5s %-8s %-8s %-5s', '#', 'Action', 'DUR', 'CP', 'EQUA', 'EPRG', 'IQ');
      // logger.log('%2d %30s %5.0f %5.0f %8.1f %8.1f %5.1f', s.step, '', s.durabilityState, s.cpState, s.qualityState, s.progressState, 0);

    }
    for (let i = 0; i < individual.length; i++) {
    // var action = individual[i];

    // Ranged edit -- Combo actions. Basically do everything twice over if there's a combo action. Woo.
    let actionsArray = vec![];

    if (individual[i].isCombo){
    actionsArray[0] = getComboAction(individual[i].comboName1);
    actionsArray[1] = getComboAction(individual[i].comboName2);
    } else {
    actionsArray[0] = individual[i];
    }
    for (var j = 0; j < actionsArray.length; j++) {
    var action = actionsArray[j];


    // Occur regardless of dummy actions
    //==================================
    state.step += 1;

    // Condition Calculation
    var condQualityIncreaseMultiplier = 1;
    if (!ignoreConditionReq) {
    condQualityIncreaseMultiplier *= (ppNormal + 1.5 * ppGood * Math.pow(1 - (ppGood + pGood) / 2, state.synth.maxTrickUses) + 4 * ppExcellent + 0.5 * ppPoor);
    }

    // Calculate Progress, Quality and Durability gains and losses under effect of modifiers
    var r = ApplyModifiers(state, action, SimCondition);

    // Calculate final gains / losses
    var successProbability = r.successProbability;
    if (assume_success) {
    successProbability = 1;
    }
    var progressGain = r.bProgressGain;
    if (progressGain > 0) {
    state.reliability = state.reliability * successProbability;
    }

    var qualityGain = condQualityIncreaseMultiplier * r.bQualityGain;

    // Floor gains at final stage before calculating expected value
    progressGain = successProbability * Math.floor(progressGain);
    qualityGain = successProbability * Math.floor(qualityGain);

    // Occur if a wasted action
    //==================================
    if (((state.progressState >= state.synth.recipe.difficulty) || (state.durabilityState <= 0) || (state.cpState < 0)) && (action != AllActions.dummyAction)) {
    state.wastedActions += 1;
    }

    // Occur if not a wasted action
    //==================================
    else {

    UpdateState(state, action, progressGain, qualityGain, r.durabilityCost, r.cpCost, SimCondition, successProbability);

    // Ending condition update
    if (!ignoreConditionReq) {
    ppPoor = ppExcellent;
    ppGood = pGood * ppNormal;
    ppExcellent = pExcellent * ppNormal;
    ppNormal = 1 - (ppGood + ppExcellent + ppPoor);
    }

    }

    var iqCnt = 0;
    if (AllActions.innerQuiet.shortName in state.effects.countUps) {
    iqCnt = state.effects.countUps[AllActions.innerQuiet.shortName];
    }
    if (debug) {
    logger.log('%2d %30s %5.0f %5.0f %8.1f %8.1f %5.1f %8.1f %8.1f %5.0f %5.0f %5.0f', state.step, action.name, state.durabilityState, state.cpState, state.qualityState, state.progressState, iqCnt, r.control, qualityGain, Math.floor(r.bProgressGain), Math.floor(r.bQualityGain), state.wastedActions);
    }
    else if (verbose) {
    logger.log('%2d %30s %5.0f %5.0f %8.1f %8.1f %5.1f', state.step, action.name, state.durabilityState, state.cpState, state.qualityState, state.progressState, iqCnt);
    }

    state.action = action.shortName
    }

    }

    // Check for feasibility violations
    var chk = state.checkViolations();

    if (debug) {
    logger.log('Progress Check: %state, Durability Check: %state, CP Check: %state, Tricks Check: %state, Reliability Check: %state, Wasted Actions: %d', chk.progressOk, chk.durabilityOk, chk.cpOk, chk.trickOk, chk.reliabilityOk, state.wastedActions);
    }
    else if (verbose) {
    logger.log('Progress Check: %state, Durability Check: %state, CP Check: %state, Tricks Check: %state, Reliability Check: %state, Wasted Actions: %d', chk.progressOk, chk.durabilityOk, chk.cpOk, chk.trickOk, chk.reliabilityOk, state.wastedActions);
    }

    // Return final state
    state.action = individual[individual.length-1].shortName;
    return state;

}*/


#[wasm_bindgen]
pub fn run_sim(individual: &JsValue) {

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[wasm_bindgen]
pub fn simulator_main(recipe: &JsValue) {
    let recipe: Recipe = recipe.into_serde().unwrap();
}
