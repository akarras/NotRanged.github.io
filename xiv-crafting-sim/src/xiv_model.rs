use crate::actions::{Action, ActionType};
use crate::level_table;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use crate::effect_tracker::EffectData;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Crafter {
    // pub(crate) cls: String,
    pub(crate) craftsmanship: u32,
    pub(crate) control: u32,
    #[serde(rename = "cp")]
    pub(crate) craft_points: u32,
    pub(crate) level: u32,
    #[serde(default)]
    pub(crate) specialist: bool,
    pub actions: Vec<Action>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    pub(crate) base_level: u32,
    pub(crate) level: u32,
    pub(crate) difficulty: u32,
    pub(crate) durability: u32,
    pub(crate) safety_margin: u32,
    pub(crate) start_quality: u32,
    pub(crate) max_quality: u32,
    pub(crate) suggested_craftsmanship: u32,
    pub(crate) suggested_control: u32,
    pub(crate) progress_divider: f64,
    pub(crate) progress_modifier: Option<u32>,
    pub(crate) quality_divider: f64,
    pub(crate) quality_modifier: Option<u32>,
    pub(crate) stars: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SolverVars {
    pub(crate) solve_for_completion: bool,
    #[serde(rename = "remainderCPFitnessValue")]
    pub(crate) remainder_cp_fitness_value: i32,
    pub(crate) remainder_dur_fitness_value: i32,
    pub(crate) max_stagnation_counter: i32,
    pub(crate) population: i32,
    pub(crate) generations: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Synth {
    pub(crate) crafter: Crafter,
    pub(crate) recipe: Recipe,
    #[serde(default)]
    pub(crate) max_trick_uses: i32,
    pub(crate) reliability_percent: u32,
    pub(crate) max_length: u32,
    #[serde(rename = "solver")]
    pub(crate) solver_vars: SolverVars,
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
            (base_value * (self.recipe.quality_modifier.unwrap_or(100) as f64) / 100.0).floor()
                as u32
        } else {
            base_value as u32
        }
    }
}

pub type AbilityMap = EffectData;

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Effects {
    count_downs: AbilityMap,
    count_ups: AbilityMap,
    // still used?
    // indefinites: AbilityMap,
}

impl Display for Effects {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "effects: ")?;
        write!(f, "CD's: [{}]", self.count_downs)?;
        write!(f, "CU's: [{}]", self.count_ups)?;
        write!(f, "")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub(crate) enum Condition {
    Poor,
    Normal,
    Good,
    Excellent
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Poor => write!(f, "Poor"),
            Condition::Normal => write!(f, "Normal"),
            Condition::Good => write!(f, "Good"),
            Condition::Excellent => write!(f, "Excellent")
        }
    }
}

impl Condition {
    fn check_good_or_excellent(&self) -> bool {
        match self {
            Condition::Good => true,
            Condition::Excellent => true,
            _ => false
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct State<'a> {
    pub synth: &'a Synth,
    pub step: u32,
    pub last_step: u32,
    pub action: Option<Action>, // Action leading to this state
    pub durability_state: i32,
    pub cp_state: i32,
    pub bonus_max_cp: i32,
    pub quality_state: i32,
    pub progress_state: i32,
    pub wasted_actions: f64,
    pub trick_uses: i32,
    pub name_of_element_uses: i32,
    pub reliability: i32,
    pub effects: Effects,
    pub condition: Condition,

    // Advancedtouch combo stuff
    pub touch_combo_step: i32,

    // Internal state variables set after each step.
    pub iq_cnt: i32,
    pub control: i32,
    pub quality_gain: i32,
    pub base_progress_gain: u32,
    pub base_quality_gain: u32, // Rustversion: for some reason these are almost the same name?
    pub success: bool,
}

impl Default for Condition {
    fn default() -> Self {
        Self::Normal
    }
}

impl Display for State<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:>2} {:>4}/{:>4} qual: {:>4}/{:>4} dur: {:>3}/{:>3} cp: {:>4}/{:>4} BP: {:>4} BQ: {:>4} action: {:?} effects: {} cond: {}", self.step,
               self.progress_state, self.synth.recipe.difficulty,
               self.quality_state, self.synth.recipe.max_quality,
               self.durability_state, self.synth.recipe.durability, self.cp_state, self.synth.crafter.craft_points, self.base_progress_gain, self.base_quality_gain, self.action, self.effects, self.condition)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Violations {
    pub progress_ok: bool,
    pub cp_ok: bool,
    pub durability_ok: bool,
    pub trick_ok: bool,
    pub reliability_ok: bool,
}

impl Violations {
    pub fn is_okay(&self) -> bool {
        self.durability_ok && self.cp_ok && self.reliability_ok && self.trick_ok && self.progress_ok
    }
}

impl State<'_> {
    pub(crate) fn check_violations(&self) -> Violations {
        let progress_ok = self.progress_state >= self.synth.recipe.difficulty as i32;
        let cp_ok = self.cp_state >= 0;
        let mut durability_ok = false;
        if self.durability_state >= -5
        // self.progress_state >= self.synth.recipe.difficulty as i32 why tho?
        {
            if let Some(action) = self.action {
                let details = action.details();
                if details.durability_cost == 10 || self.durability_state >= 0 {
                    durability_ok = true
                }
            }

            if self.durability_state >= 0 {
                durability_ok = true;
            }
        }

        let trick_ok = self.trick_uses <= self.synth.max_trick_uses;
        let reliability_ok = self.reliability > self.synth.reliability_percent as i32 / 100;
        Violations {
            progress_ok,
            cp_ok,
            durability_ok,
            trick_ok,
            reliability_ok,
        }
    }

    /// Returns an int with a penality
    pub fn calculate_penalties(&self, penality_weight: f64) -> f64 {
        let violations = self.check_violations();
        let mut penalties = self.wasted_actions / 20.0;

        if !violations.durability_ok {
            penalties += self.durability_state.abs() as f64;
        }

        if !violations.progress_ok {
            penalties += (self.synth.recipe.difficulty as i32 - self.progress_state).abs() as f64
        }

        if !violations.cp_ok {
            penalties += self.cp_state.abs() as f64
        }

        if self.trick_uses > self.synth.max_trick_uses {
            penalties += (self.trick_uses - self.synth.max_trick_uses).abs() as f64
        }

        if self.reliability < (self.synth.reliability_percent / 100) as i32 {
            penalties += ((self.synth.reliability_percent / 100) as i32 - self.reliability) as f64
        }

        penalties * penality_weight
    }
}

impl<'a> From<&'a Synth> for State<'a> {
    fn from(synth: &'a Synth) -> Self {
        State {
            synth, // TODO this could be a parent ref, PhantomData stuff.
            step: 0,
            last_step: 0,
            action: None,
            effects: Effects {
                count_ups: [(Action::InnerQuiet, -1)].into_iter().collect(),
                ..Default::default()
            },
            reliability: 1,
            cp_state: synth.crafter.craft_points as i32,
            bonus_max_cp: 0,
            quality_state: synth.recipe.start_quality as i32,
            progress_state: 0,
            wasted_actions: 0.0,
            trick_uses: 0,
            condition: Condition::Normal,
            touch_combo_step: 0,
            iq_cnt: 0,
            control: 0,
            quality_gain: 0,
            base_progress_gain: 0,
            base_quality_gain: 0,
            durability_state: synth.recipe.durability as i32,
            name_of_element_uses: 0,
            success: false
        }
    }
}

impl Synth {
    fn prob_good_for_synth(&self) -> f64 {
        let recipe_level = self.recipe.level;
        let quality_assurance = self.crafter.level >= 63;
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

    fn prob_excellent_for_synth(&self) -> f64 {
        let recipe_level = self.recipe.level;
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

    fn get_effective_crafter_level(&self) -> u32 {
        let eff_crafter_level = self.crafter.level;
        level_table::level_table_lookup(eff_crafter_level)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModifierResult {
    craftsmanship: u32,
    control: u32,
    eff_crafter_level: u32,
    eff_recipe_level: u32,
    level_difference: i32,
    success_probability: f64,
    quality_increase_multiplier: f64,
    progress_gain: u32,
    quality_gain: u32,
    durability_cost: f64,
    cp_cost: i32,
}

/// I could just do the functions that the JS uses, but I have lifetimes to worry about.
pub(crate) enum SimulationCondition {
    Simulation {
        ignore_condition: bool,
        pp_poor: f64,
        pp_normal: f64,
        pp_good: f64,
        pp_excellent: f64,
    },
}

impl SimulationCondition {
    pub(crate) fn new_sim_condition() -> SimulationCondition {
        SimulationCondition::Simulation {
            ignore_condition: true,
            pp_poor: 0.0,
            pp_normal: 1.0,
            pp_good: 0.0,
            //ignore_condition_req: false,
            pp_excellent: 0.0,
        }
    }

    fn update(&mut self, p_good: f64, p_excellent: f64) {
        match self {
            SimulationCondition::Simulation {
                ignore_condition,
                pp_poor,
                pp_normal,
                pp_good,
                pp_excellent,
            } => {
                if !*ignore_condition {
                    *pp_poor = *pp_excellent;
                    *pp_good = p_good * *pp_normal;
                    *pp_excellent = p_excellent * *pp_normal;
                    *pp_normal = 1.0 - (*pp_good + *pp_excellent + *pp_poor);
                }
            }
        }
    }

    fn check_good_or_excellent(&self, state: &State) -> bool {
        match self {
            SimulationCondition::Simulation { .. } => true,
        }
    }

    fn p_good_or_excellent(&self) -> f64 {
        match self {
            SimulationCondition::Simulation {
                ignore_condition,
                pp_excellent,
                pp_good,
                ..
            } => {
                if *ignore_condition {
                    1.0
                } else {
                    pp_excellent + pp_good
                }
            }
        }
    }
}

impl<'a> State<'a> {
    fn apply_modifiers(
        &mut self,
        action: Action,
        condition: &SimulationCondition,
    ) -> ModifierResult {
        let craftsmanship = self.synth.crafter.craftsmanship;
        let mut control = self.synth.crafter.control;
        let mut cp_cost = action.details().cp_cost;

        // Effects modifying level difference
        let eff_crafter_level = self.synth.get_effective_crafter_level();
        let eff_recipe_level = self.synth.recipe.level;
        let level_difference = eff_crafter_level as i32 - eff_recipe_level as i32;
        // let original_level_difference = eff_crafter_level - eff_recipe_level;
        let pure_level_difference =
            self.synth.crafter.level as i32 - self.synth.recipe.base_level as i32;
        // let recipe_level = eff_recipe_level;

        // Effects modifying probability
        let action_details = action.details();
        let mut success_probability = action_details.success_probability;
        if action.eq(&Action::FocusedSynthesis) || action.eq(&Action::FocusedTouch) {
            if let Some(sa) = &self.action {
                if sa.eq(&Action::Observe) {
                    success_probability = 1.0;
                }
            }
        }

        success_probability = success_probability.min(1.0);

        // Advanced Touch Combo
        if action.eq(&Action::AdvancedTouch) {
            if let Some(sa) = &self.action {
                if *sa == Action::StandardTouch && self.touch_combo_step == 1 {
                    self.touch_combo_step = 0;
                    cp_cost = 18;
                }
            }
        }
        // Add combo bonus following Basic Touch
        if action.eq(&Action::StandardTouch) {
            if let Some(sa) = &self.action {
                if *sa == Action::BasicTouch {
                    cp_cost = 18;
                    self.wasted_actions -= 0.05;
                    self.touch_combo_step = 1;
                }
                if *sa == Action::StandardTouch {
                    self.wasted_actions += 0.1;
                }
            }
        }

        // Penalize use of WasteNot during solveforcompletion runs

        if (action == Action::WasteNot
            || action == Action::WasteNot2) && self.synth.solver_vars.solve_for_completion
        {
            self.wasted_actions += 50.0;
        }

        // Effects modifying progress increase multiplier
        let mut progress_increase_multiplier = 1.0;

        if (action_details.progress_increase_multiplier > 0.0)
            && (self.effects.count_downs.get(Action::MuscleMemory).is_some())
        {
            progress_increase_multiplier += 1.0;
            self.effects.count_downs.remove(Action::MuscleMemory);
        }

        if self.effects.count_downs.get(Action::Veneration).is_some() {
            progress_increase_multiplier += 0.5;
        }

        if action.eq(&Action::MuscleMemory) {
            if self.step != 1 {
                self.wasted_actions += 10.0;
                progress_increase_multiplier = 0.0;
                cp_cost = 0;
            }
        }

        // Effects modifying quality increase multiplier
        let mut quality_increase_multiplier = 1.0;
        let mut quality_increase_multiplier_iq = 1.0; // This is calculated seperately because it's multiplicative instead of additive! See: how teamcrafting does it

        if self.effects.count_downs.get(Action::GreatStrides).is_some()
            && quality_increase_multiplier > 0.0
        {
            quality_increase_multiplier += 1.0;
        }

        if self.effects.count_downs.get(Action::Innovation).is_some() {
            quality_increase_multiplier += 0.5;
        }

        if let Some((_, inner_quiet_value)) = self.effects.count_ups.get(Action::InnerQuiet) {
            quality_increase_multiplier_iq += 0.1 * (*inner_quiet_value + 1) as f64
            // +1 because buffs start incrementing from 0
        }

        // We can only use Byregot actions when we have at least 1 stacks of inner quiet
        if action == Action::ByregotsBlessing {
            let num_inner_quiets = self
                .effects
                .count_ups
                .get(Action::InnerQuiet)
                .map(|(_, i)| *i)
                .unwrap_or(0);
            if num_inner_quiets >= 1
            {
                quality_increase_multiplier *= 1.0 + (0.2 * (num_inner_quiets + 1) as f64).min(3.0);
            } else {
                quality_increase_multiplier = 0.0;
            }
        }

        // Calculate base and modified progress gain
        let mut progress_gain = self
            .synth
            .calculate_base_progress_increase(eff_crafter_level, craftsmanship);

        progress_gain = (progress_gain as f64
            * action_details.progress_increase_multiplier
            * progress_increase_multiplier) as u32;

        // Calculate base and modified quality gain
        let mut quality_gain = self
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
            if self
                .effects
                .count_ups
                .get(Action::InnerQuiet)
                .map(|(_, m)| *m)
                .unwrap_or(0)
                != 9
            {
                self.wasted_actions += 1.0;
                quality_gain = 0;
            }
        }

        // Effects modifying durability cost
        let mut durability_cost = action_details.durability_cost as f64;
        if self.effects.count_downs.get(Action::WasteNot).is_some()
            || self.effects.count_downs.get(Action::WasteNot2).is_some()
        {
            if action.eq(&Action::PrudentTouch) {
                quality_gain = 0;
                self.wasted_actions += 1.0;
            } else if action.eq(&Action::PrudentSynthesis) {
                progress_gain = 0;
                self.wasted_actions += 1.0;
            } else {
                durability_cost *= 0.5;
            }
        }

        if self.durability_state < durability_cost as i32 {
            if action == Action::Groundwork || action == Action::Groundwork2 {
                progress_increase_multiplier *= 0.5;
            }
        }

        // Effects modifying quality gain directly
        if action.eq(&Action::TrainedEye) {
            if self.step == 1 && pure_level_difference >= 10 && self.synth.recipe.stars.is_none() {
                quality_gain = self.synth.recipe.max_quality;
            } else {
                self.wasted_actions += 1.0;
                quality_gain = 0;
                cp_cost = 0;
            }
        }

        // We can only use Precise Touch when state material condition is Good or Excellent. Default is true for probabilistic method.
        if action.eq(&Action::PreciseTouch) {
            if condition.check_good_or_excellent(self) {
                quality_gain *= condition.p_good_or_excellent() as u32;
            } else {
                self.wasted_actions += 1.0;
                quality_gain = 0;
                cp_cost = 0;
            }
        }

        if action.eq(&Action::Reflect) {
            if self.step != 1 {
                self.wasted_actions += 1.0;
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
        }
    }

    fn use_conditional_action(&mut self, condition: &SimulationCondition) -> bool {
        if self.cp_state > 0 && condition.check_good_or_excellent(self) {
            self.trick_uses += 1;
            return true;
        } else {
            self.wasted_actions += 1.0;
            return false;
        }
    }

    fn apply_special_action_effects(&mut self, action: Action, condition: &SimulationCondition) {
        // STEP_02
        // Effect management
        //==================================
        // Special Effect Actions
        if action == Action::MastersMend {
            self.durability_state += 30;
            if self.synth.solver_vars.solve_for_completion {
                self.wasted_actions += 50.0; // Bad code, but it works. We don't want dur increase in solveforcompletion.
            }
        }

        if self.effects.count_downs.get(Action::Manipulation).is_some()
            && self.durability_state > 0
            && action != Action::Manipulation
        {
            self.durability_state += 5;
            if self.synth.solver_vars.solve_for_completion {
                self.wasted_actions += 50.0; // Bad code, but it works. We don't want dur increase in solveforcompletion.
            }
        }

        if action == Action::ByregotsBlessing {
            if self.effects.count_ups.get(Action::InnerQuiet).is_some() {
                self.effects.count_ups.remove(Action::InnerQuiet);
            } else {
                self.wasted_actions += 1.0;
            }
        }

        if action == Action::Reflect {
            if self.step == 1 {
                if let Some(count) = self.effects.count_ups.get_mut(Action::InnerQuiet) {
                    *count = 1;
                } else {
                    self.effects.count_ups.insert(Action::InnerQuiet, 0); // what does this even get inserted as?
                }
            } else {
                self.wasted_actions += 1.0;
            }
        }

        if self.effects.count_downs.get(Action::FinalAppraisal).is_some() {
            self.progress_state = self.progress_state.clamp(0, (self.synth.recipe.difficulty - 1) as i32);
        }

        let action_details = action.details();
        if action_details.quality_increase_multiplier > 0.0
            && self.effects.count_downs.get(Action::GreatStrides).is_some()
        {
            self.effects.count_downs.remove(Action::GreatStrides);
        }

        // Manage effects with conditional requirements
        if action_details.on_excellent || action_details.on_good {
            if self.use_conditional_action(condition) {
                if action == Action::TricksOfTheTrade {
                    self.cp_state += (20.0 * condition.p_good_or_excellent()) as i32;
                }
            }
        }

        /*if action == Action::Veneration
            && self.effects.count_downs.get(Action::Veneration).is_some()
        {
            self.wasted_actions += 1.0
        }
        if action == Action::Innovation
            && self.effects.count_downs.get(Action::Innovation).is_some()
        {
            self.wasted_actions += 1.0
        }*/
    }

    fn update_effects_counters(
        &mut self,
        action: Action,
        condition: &SimulationCondition,
        success_probability: f64,
    ) {
        // STEP_03
        // Countdown / Countup Management
        //===============================
        // Decrement countdowns
        let action_details = action.details();
        for value in self.effects.count_downs.iter_mut() {
            let mut is_valid = true;
            if let Some((_, count)) = value {
                *count -= 1;
                if *count <= 0 {
                    is_valid = false;
                }
            }
            if !is_valid {
                *value = None;
            }
        }
        if self.effects.count_ups.get(Action::InnerQuiet).is_some() {
            // Increment inner quiet countups that have conditional requirements
            if action == Action::PreparatoryTouch {
                if let Some(quiet) = self.effects.count_ups.get_mut(Action::InnerQuiet) {
                    *quiet += 2;
                }
            }
            // Increment inner quiet countups that have conditional requirements
            else if action == Action::PreciseTouch && condition.check_good_or_excellent(self) {
                let quiet_increment =
                    (2.0 * success_probability * condition.p_good_or_excellent()) as i32;
                if let Some(quiet) = self.effects.count_ups.get_mut(Action::InnerQuiet) {
                    *quiet += quiet_increment as i8;
                }
            }
            // Increment all other inner quiet count ups
            else if action.details().quality_increase_multiplier > 0.0
                && action != Action::Reflect
                && action != Action::TrainedFinesse
            {
                if let Some(quiet) = self.effects.count_ups.get_mut(Action::InnerQuiet) {
                    *quiet += (1.0 * success_probability) as i8;
                }
            }

            // Cap inner quiet stacks at 9 (10)
            if let Some(quiet) = self.effects.count_ups.get_mut(Action::InnerQuiet) {
                *quiet = (*quiet).min(9);
            }
        }

        // Initialize new effects after countdowns are managed to reset them properly
        if action_details.action_type == ActionType::CountUp {
            self.effects.count_ups.insert(action, 0);
        }

        if let ActionType::Countdown { active_turns } = action_details.action_type {
            if action == Action::MuscleMemory && self.step != 1 {
                self.wasted_actions += 1.0;
            } else {
                self.effects.count_downs.insert(action, active_turns as i8);
            }
        }
    }
    fn update_state(
        &mut self,
        action: Action,
        progress_gain: i32,
        quality_gain: i32,
        durability_cost: i32,
        cp_cost: i32,
        condition: &SimulationCondition,
        success_probability: f64,
    ) {
        // State tracking
        self.progress_state += progress_gain;
        self.quality_state += quality_gain;
        self.durability_state -= durability_cost;
        self.cp_state -= cp_cost;
        self.last_step += 1;
        self.apply_special_action_effects(action, condition);
        self.update_effects_counters(action, condition, success_probability);

        // Sanity checks for state variables
        /* Removing this for solveForCompletion, hopefully it doesn't cause issues! :)
        if ((s.durabilityState >= -5) && (s.progressState >= s.synth.recipe.difficulty)) {
            //s.durabilityState = 0;
        }
        */
        self.durability_state = self
            .durability_state
            .min(self.synth.recipe.durability as i32);
        self.cp_state = self
            .cp_state
            .min(self.synth.crafter.craft_points as i32 + self.bonus_max_cp);
    }
    pub(crate) fn add_action(&self, action: Action, sim_condition: &mut SimulationCondition) -> State<'a> {
        let mut state = self.clone();
        state.step += 1;
        // TODO figure out how to handle simulation condition *better*
        let p_good = self.synth.prob_good_for_synth();
        let p_excellent = self.synth.prob_excellent_for_synth();

        let mut condition_quality_increase_multiplier = 1.0;
        match sim_condition {
            SimulationCondition::Simulation { ignore_condition, pp_poor, pp_normal, pp_good, pp_excellent } => {
                if !*ignore_condition {
                    condition_quality_increase_multiplier *= *pp_normal + 1.5 * *pp_good * (1.0 - (*pp_good + p_good) / 2.0).powf(state.synth.max_trick_uses as f64) + 4.0 * *pp_excellent + 0.5 * *pp_poor;
                }
            }
        }

        let result = state.apply_modifiers(
            action,
            sim_condition,
        );
        state.base_quality_gain = result.quality_gain;
        state.base_progress_gain = result.progress_gain;
        // Calculate final gains / losses
        let success_probability = result.success_probability;
        // no assume success for now
        let mut progress_gain = result.progress_gain;
        if progress_gain > 0 {
            state.reliability = (state.reliability as f64 * success_probability) as i32;
        }

        progress_gain = (success_probability * progress_gain as f64) as u32;
        //// Floor gains at final stage before calculating expected value
        let mut quality_gain = condition_quality_increase_multiplier * result.quality_gain as f64;
        quality_gain = success_probability * quality_gain.floor();


        state.update_state(
            action,
            progress_gain as i32,
            quality_gain as i32,
            result.durability_cost as i32,
            result.cp_cost,
            sim_condition,
            result.success_probability,
        );

        sim_condition.update(p_good, p_excellent);
        state.action = Some(action);
        state
    }
}

#[cfg(test)]
mod test {
    use crate::xiv_model::{Synth, State, SimulationCondition};
    use crate::actions::Action;

    const CRAFTER_SYNTH : &str = r#"{"crafter":{"level":54,"craftsmanship":285,"control":249,"cp":309,"actions":["muscleMemory","basicSynth2","basicTouch","standardTouch","byregotsBlessing","preciseTouch","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Culinarian","level":40,"difficulty":138,"durability":60,"startQuality":0,"maxQuality":3500,"baseLevel":40,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":68,"suggestedCraftsmanship":136,"name":"Grade 4 Skybuilders' Sesame Cookie"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":50},"debug":true}"#;

    #[test]
    fn basic_action_sim() {
        let synth : Synth = serde_json::from_str(CRAFTER_SYNTH).unwrap();
        let mut simulation_condition = SimulationCondition::Simulation {
            ignore_condition: true,
            pp_poor: 0.0,
            pp_normal: 1.0,
            pp_good: 0.0,
            pp_excellent: 0.0
        };
        let mut state : State = (&synth).into();
        state = state.add_action(Action::MuscleMemory, &mut simulation_condition);
        let quality_touch = state.add_action(Action::StandardTouch, &mut simulation_condition);
        println!("q touch state {}", quality_touch);
        //assert_eq!(quality_touch.quality_gain, 140);
        assert_eq!(quality_touch.quality_state, 147);
        let progress_touch = state.add_action(Action::BasicSynth, &mut simulation_condition);
        assert_eq!(progress_touch.progress_state, 177);
    }
    #[test]
    fn match_site() {
        let synth : Synth = serde_json::from_str(CRAFTER_SYNTH).unwrap();
        let simulation_condition = SimulationCondition::Simulation {
            ignore_condition: true,
            pp_poor: 0.0,
            pp_normal: 1.0,
            pp_good: 0.0,
            pp_excellent: 0.0
        };
        let mut state : State = (&synth).into();
        let mut cond = SimulationCondition::new_sim_condition();
        for action in [Action::BasicSynth2, Action::Innovation, Action::BasicTouch, Action::StandardTouch, Action::BasicTouch, Action::StandardTouch, Action::MastersMend, Action::Innovation, Action::BasicTouch, Action::StandardTouch, Action::GreatStrides, Action::ByregotsBlessing, Action::BasicSynth2] {
            state = state.add_action(action, &mut cond);
            println!("{}", state);
        }
        println!("FINAL: {}", state);
        assert_eq!(state.progress_state, 140);
        assert_eq!(state.quality_state, 2535);

    }
}

