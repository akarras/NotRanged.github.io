use xiv_crafting_sim::simulator::CrafterActions;
use xiv_crafting_sim::xiv_model::Synth;
use genevo::genetic::FitnessFunction;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{SeedableRng, Rng};


fn f_calc(synth: &Synth, target_actions: &CrafterActions) -> i32 {
    synth.fitness_of(&target_actions)
}

fn fitness(c: &mut Criterion) {
    let synth : Synth = serde_json::from_str(r#"{"crafter":{"level":51,"craftsmanship":117,"control":158,"cp":180,"actions":["basicSynth2","basicTouch","standardTouch","byregotsBlessing","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Culinarian","level":40,"difficulty":138,"durability":60,"startQuality":0,"maxQuality":3500,"baseLevel":40,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":68,"suggestedCraftsmanship":136,"name":"Grade 4 Skybuilders' Sesame Cookie"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#).unwrap();
    let mut dummy = vec![1,2,3,0,4,5,10,2,3,4,5,6,7,8];

    c.bench_function("fitness", |b| b.iter(|| f_calc(&synth, black_box(&dummy))));
    let mut rng = rand::rngs::SmallRng::from_entropy();
    c.bench_function("fitness_rand", |b| b.iter(||
        for _i in 0..10000 {
            for value in &mut dummy {
                *value = rng.gen_range(0..6);
            }
            f_calc(&synth, black_box(&dummy));
        }));
}

criterion_group!{
    name = fitness_bench;
    config = Criterion::default().significance_level(0.1).sample_size(10000);
    targets = fitness
}
criterion_main!(fitness_bench);