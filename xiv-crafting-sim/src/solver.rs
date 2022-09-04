use std::cmp::Ordering;
use crate::actions::{Action, ActionType};
use crate::xiv_model::{State, Synth};
#[cfg(feature = "thread")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Attempts to use brute force to find *the* definitively best solution
/// Will find the shortest synth possible that achieves it's goals the highest quality
/// # Arguments
/// * synth - Synth you want to solve for
/// * goals - Goals of the synth

pub fn find_solution(synth: &Synth) -> Vec<Action> {
    let state = State::from(synth);
    better_options(state, &synth)
        .map(|(mut a, s)| {
            a.reverse();
            println!("{a:?} {s:?}");
            a
        })
        .unwrap_or_default() // default none = no actions were found
}


fn add_action<'a>(action: Action, state: &State<'a>, synth: &'a Synth) -> Option<(Vec<Action>, State<'a>)> {
    if action == Action::TricksOfTheTrade {
        return None;
    }

    let new_state = state.add_action(action);
    // determine whether this is a complete state
    if new_state.progress_state >= state.synth.recipe.difficulty as i32 {
        let actions = vec![action];
        return Some((actions, new_state));
    }
    // wasted actions for this are pointless.
    if new_state.wasted_actions > 0.0 {
        return None;
    }
    if new_state.reliability < 0 {
        return None;
    }
    if new_state.durability_state <= 0 {
        return None;
    }
    // invalid states should return None
    if new_state.cp_state < 0 {
        return None;
    }
    // now try children of THIS action
    let (mut actions, state) = better_options(new_state, synth)?;
    actions.push(action);
    Some((actions, state))
}


fn filter_usable(action: Action, state: &State) -> bool {
    let action = action;
    match action {
        Action::Observe => return false,
        Action::MuscleMemory => {
            if state.step > 1 {
                return false;
            }
        }
        Action::Reflect => {
            if state.step > 1 {
                return false;
            }
        }
        Action::TrainedFinesse => return false,

        _ => {}
    }
    // try to filter actions that don't make sense
    let details = action.details();
    // only rely on full success for solver
    if details.success_probability < 1.0 {
        return false;
    }

    if details.on_excellent || details.on_good {
        return false;
    }

    if details.cp_cost < 5 && details.durability_cost < 10 {
        if let Some(a) = state.action {
            if action == a {
                return false;
            }
        }
    }

    let need_quality = state.synth.recipe.max_quality >= state.quality_state as u32;
    // if our state has quality, we only need progress
    if need_quality
        && details.action_type == ActionType::Immediate
        && details.progress_increase_multiplier == 0.0
    {
        // quality is full, and this action increases progress
        return false;
    }
    // if we have a countdown active, and the action has more than two turns, skip this
    if let ActionType::Countdown { active_turns } = details.action_type {
        if let Some((_, turns)) = state.effects.count_downs.get(action) {
            if *turns > 1 as i8 {
                return false;
            }
        }
    }

    if state.step >= state.synth.max_length {
        return false;
    }

    true
}


fn compare_actions<'r, 's>((a_actions, a_state): &'r (Vec<Action>, State), (b_actions, b_state): &'s (Vec<Action>, State)) -> Ordering {
    // assume all states complete progress
    if a_state.quality_state >= a_state.synth.recipe.max_quality as i32
        && b_state.quality_state >= a_state.synth.recipe.max_quality as i32
    {
        // both states have max quality, so check if one has less steps
        a_state.step.cmp(&b_state.step)
    } else {
        // both qualities aren't complete, choose the higher quality synth
        a_state.quality_state.cmp(&b_state.quality_state)
    }
}

/// internal worker that recursively checks tries each possible action
#[cfg(feature = "thread")]
fn better_options<'a>(
    state: State<'a>,
    synth: &'a Synth,
) -> Option<(Vec<Action>, State<'a>)> {
    // advance over iter
    synth
        .crafter
        .actions
        .par_iter()
        // filtering out actions that don't have any effect can help the most
        .filter(|action| {
            filter_usable(**action, &state)
        })
        .flat_map(|a| {
            add_action(*a, &state, synth)
        })
        .max_by(compare_actions)
}

/// internal worker that recursively checks tries each possible action
#[cfg(not(feature = "thread"))]
fn better_options<'a>(
    state: State<'a>,
    synth: &'a Synth,
) -> Option<(Vec<Action>, State<'a>)> {
    // advance over iter
    synth
        .crafter
        .actions
        .iter()
        // filtering out actions that don't have any effect can help the most
        .filter(|action| {
            filter_usable(**action, &state)
        })
        .flat_map(|a| {
            add_action(*a, &state, synth)
        })
        .max_by(compare_actions)
}

#[cfg(test)]
mod test {
    use crate::solver::find_solution;
    use crate::xiv_model::Synth;

    const TEST_SYNTH: &str = r#"{"crafter":{"level":78,"craftsmanship":863,"control":877,"cp":412,"actions":["muscleMemory","reflect","basicSynth2","carefulSynthesis","groundwork","intensiveSynthesis","delicateSynthesis","basicTouch","standardTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","finalAppraisal","observe"]},"recipe":{"cls":"Weaver","level":390,"difficulty":1195,"durability":60,"startQuality":0,"safetyMargin":0,"maxQuality":3010,"baseLevel":71,"progressDivider":101,"progressModifier":100,"qualityDivider":81,"qualityModifier":100,"suggestedControl":1220,"suggestedCraftsmanship":1320,"name":"Custom Gathering Tool Components"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":50,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":12000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":2000},"debug":true}"#;

    #[test]
    fn test() {
        let synth: Synth = serde_json::from_str(TEST_SYNTH).unwrap();
        let solution = find_solution(&synth);
        assert!(solution.len() > 0);
        assert_eq!(solution, vec![]);
        eprintln!("{solution:?}");
    }
}
