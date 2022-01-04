use crate::level_table::level_table_lookup;
use crate::xiv_model::Synth;
use genevo::population::ValueEncodedGenomeBuilder;
use genevo::prelude::{GenomeBuilder, Rng};
use genevo::random::SampleUniform;
use std::fmt::Debug;

/// Constructs a genome of crafter actions in a hopefully intelligent way
pub struct CraftActionGenomeBuilder<V> {
    min_length: usize,
    max_length: usize,
    min_value: V,
    max_value: V,
}

impl<V> CraftActionGenomeBuilder<V> {
    pub(crate) fn new(synth: &Synth, min_value: V, max_value: V) -> Self {
        let difficulty = synth.recipe.difficulty;
        let max_quality = synth.recipe.max_quality;
        let eff_crafter_level = level_table_lookup(synth.crafter.level);
        let base_progress =
            synth.calculate_base_progress_increase(eff_crafter_level, synth.crafter.craftsmanship);
        let base_quality =
            synth.calculate_base_quality_increase(eff_crafter_level, synth.crafter.control);
        // estimate how many steps it will take
        let prog_steps = difficulty / base_progress;
        let qual_steps = max_quality / base_quality;
        // now give a +- range of 5
        let step_range = prog_steps + qual_steps;
        // do minus operation as a signed int and max to 0 to prevent wrapping
        let min_length = (step_range as i32 - 5).max(0) as usize;
        let max_length = (step_range + 20) as usize;
        Self {
            min_length,
            max_length,
            min_value,
            max_value,
        }
    }
}

impl<V> GenomeBuilder<Vec<V>> for CraftActionGenomeBuilder<V>
where
    V: Debug + PartialEq + PartialOrd + SampleUniform + Send + Sync + Copy,
{
    fn build_genome<R>(&self, _: usize, rng: &mut R) -> Vec<V>
    where
        R: Rng + Sized,
    {
        let random_length = rng.gen_range(self.min_length..=self.max_length);
        (0..random_length)
            .map(|_| rng.gen_range(self.min_value..self.max_value))
            .collect()
    }
}
