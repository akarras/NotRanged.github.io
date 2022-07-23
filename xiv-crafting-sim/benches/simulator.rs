use criterion::{criterion_group, criterion_main, Criterion};
use xiv_crafting_sim::simulator::SimStep;
use xiv_crafting_sim::xiv_model::Synth;
use xiv_crafting_sim::CraftSimulator;
const SIM_CRAFT: &str = r#"{"crafter":{"level":90,"craftsmanship":5672,"control":2499,"cp":507,"actions":["muscleMemory","reflect","trainedEye","basicSynth2","carefulSynthesis2","groundwork2","intensiveSynthesis","prudentSynthesis","delicateSynthesis","basicTouch","standardTouch","advancedTouch","byregotsBlessing","preciseTouch","prudentTouch","preparatoryTouch","trainedFinesse","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","manipulation","veneration","greatStrides","innovation","finalAppraisal","observe"]},"recipe":{"cls":"Goldsmith","level":560,"difficulty":3500,"durability":80,"startQuality":0,"safetyMargin":0,"maxQuality":7200,"baseLevel":90,"progressDivider":130,"progressModifier":90,"qualityDivider":115,"qualityModifier":80,"suggestedControl":2635,"suggestedCraftsmanship":2805,"name":"Rarefied Chondrite Needle"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":12000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#;
fn simulator() {
    let synth : Synth = serde_json::from_str(SIM_CRAFT).unwrap();
    let mut simulator = CraftSimulator::new(synth);
    let mut step = simulator.next_generation();
    let mut generation_limit = 10;
    while let SimStep::Progress { .. } = step {
        step = simulator.next_generation();
        generation_limit -= 1;
        if generation_limit <= 0 {
            break;
        }
    }
    //println!("{:?}", step);
}

fn sim(c: &mut Criterion) {
    c.bench_function("simulator", |b| b.iter(|| simulator()));
}

criterion_group! {
    name = long_benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = sim
}
criterion_main!(long_benches);
