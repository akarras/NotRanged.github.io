use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Crafter {
    cls: u32,
    craftsmanship: u32,
    craft_points: u32,
    level: u32,
    specialist: u32,
}

#[derive(Deserialize, Serialize)]
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
    stars: u32
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all= "camelCase")]
struct SolverVars {
    solve_for_completion: bool,
    #[serde(rename = "remainderCPFitnessValue")]
    remainder_cp_fitness_value: bool,
    remainder_dur_fitness_value: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Synth {
    crafter: Crafter,
    recipe: Recipe,
}

impl Synth {
    fn calculate_base_progress_increase(&self, eff_crafter_level: u32, craftsmanship: u32) -> u32 {
        let base_value : f64 = (craftsmanship as f64 * 10.0) / self.recipe.progress_divider + 2.0;
        if eff_crafter_level <= self.recipe.level {
            (base_value * (self.recipe.progress_modifier.unwrap_or(100) as f64) / 100.0) as u32
        } else {
            base_value as u32
        }
    }

    fn calculate_base_quality_increase(&self, eff_crafter_level: u32, control: u32) -> u32 {
        let base_value : f64 = (control as f64 * 10.0) / self.recipe.quality_divider + 35.0;
        if eff_crafter_level <= self.recipe.base_level {
            (base_value * (self.recipe.quality_modifier.unwrap_or(100) as f64) / 100.0) as u32
        } else {
            base_value as u32
        }
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    synth: u32,
    step: u32,
    last_step: u32,
    action: String, // Action leading to this state
    durability_state : u32,
    cp_state : u32,
    bonus_max_cp : u32,
    quality_state : u32,
    progress_state : u32,
    wasted_actions : u32,
    trick_uses : u32,
    name_of_element_uses : u32,
    reliability : u32,
    effects : u32,
    condition : u32,

    // Advancedtouch combo stuff
    touch_combo_step : u32,

    // Internal state variables set after each step.
    iq_cnt: u32,
    control: u32,
    quality_gain: u32,
    progress_gain: bool,
    b_quality_gain: bool, // Rustversion: for some reason these are almost the same name?
    success: bool
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
    let recipe : Recipe = recipe.into_serde().unwrap();
}

