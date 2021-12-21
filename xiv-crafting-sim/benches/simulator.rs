use criterion::{criterion_group, criterion_main, Criterion};
use xiv_crafting_sim::{CraftSimulator};
use xiv_crafting_sim::simulator::SimStep;
use xiv_crafting_sim::xiv_model::Synth;

fn simulator() {
    let synth : Synth = serde_json::from_str(r#"{"crafter":{"level":51,"craftsmanship":117,"control":158,"cp":180,"actions":["basicSynth2","basicTouch","standardTouch","byregotsBlessing","tricksOfTheTrade","mastersMend","wasteNot","wasteNot2","veneration","greatStrides","innovation","observe"]},"recipe":{"cls":"Culinarian","level":40,"difficulty":138,"durability":60,"startQuality":0,"maxQuality":3500,"baseLevel":40,"progressDivider":50,"progressModifier":100,"qualityDivider":30,"qualityModifier":100,"suggestedControl":68,"suggestedCraftsmanship":136,"name":"Grade 4 Skybuilders' Sesame Cookie"},"sequence":[],"algorithm":"eaComplex","maxTricksUses":0,"maxMontecarloRuns":400,"reliabilityPercent":100,"useConditions":false,"maxLength":0,"solver":{"algorithm":"eaComplex","penaltyWeight":10000,"population":10000,"subPopulations":10,"solveForCompletion":false,"remainderCPFitnessValue":10,"remainderDurFitnessValue":100,"maxStagnationCounter":25,"generations":1000},"debug":true}"#).unwrap();
    let mut simulator = CraftSimulator::new(synth);
    let mut step = simulator.next();
    while let SimStep::Progress { .. } = step {
        step = simulator.next();
    }
    //println!("{:?}", step);
}

fn sim(c: &mut Criterion) {
    c.bench_function("simulator", |b| b.iter(|| simulator()));
}


criterion_group!{
    name = long_benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = sim
}
criterion_main!(long_benches);