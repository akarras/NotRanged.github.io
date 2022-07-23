use criterion::{black_box, criterion_group, criterion_main, Criterion};
use genevo::genetic::FitnessFunction;
use rand::{Rng, SeedableRng};
use xiv_crafting_sim::data_oriented_sim::calculate_actions;
use xiv_crafting_sim::simulator::CrafterActions;
use xiv_crafting_sim::xiv_model::Synth;

fn f_calc(synth: &Synth, target_actions: &CrafterActions) -> i32 {
    synth.fitness_of(&target_actions)
}

fn f_calc_data(synth: &Synth, target_actions: &CrafterActions) {
    let actions : Vec<_> = target_actions
        .iter()
        .flat_map(|m| synth.crafter.actions.get(*m).copied()).collect();
    let _ = calculate_actions(synth, actions);
}

fn fitness(c: &mut Criterion) {
    let synth : Synth = serde_json::from_str(r#"{"crafter":{"level":90,"craftsmanship":5672,"control":2499,"cp":507,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork2","intensiveSynthesis","prudentSynthesis","delicateSynthesis","basicTouch","standardTouch","advancedTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","trainedFinesse","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","manipulation","veneration","greatStrides","innovation","finalAppraisal","observe"]},"recipe":{"cls":"Goldsmith","level":560,"difficulty":3500,"durability":80,"startQuality":0,"safetyMargin":0,"maxQuality":7200,"baseLevel":90,"progressDivider":130,"progressModifier":90,"qualityDivider":115,"qualityModifier":80,"suggestedControl":2635,"suggestedCraftsmanship":2805,"name":"Rarefied Chondrite Needle"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":12000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#).unwrap();
    let mut dummy = vec![1, 2, 3, 0, 4, 5, 10, 2, 3, 4, 5, 6, 7, 8];

    c.bench_function("fitness", |b| b.iter(|| f_calc(&synth, black_box(&dummy))));
    c.bench_function("fitness calculated", |b| b.iter(|| f_calc_data(&synth, black_box(&dummy))));
    let mut rng = rand::rngs::SmallRng::from_entropy();
    c.bench_function("fitness_rand", |b| {
        b.iter(|| {
            for _i in 0..10000 {
                for value in &mut dummy {
                    *value = rng.gen_range(0..6);
                }
                f_calc(&synth, black_box(&dummy));
            }
        })
    });
}

criterion_group! {
    name = fitness_bench;
    config = Criterion::default().significance_level(0.1).sample_size(10000);
    targets = fitness
}
criterion_main!(fitness_bench);
