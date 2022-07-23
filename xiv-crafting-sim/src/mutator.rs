use genevo::genetic::Genotype;
use genevo::operator::{GeneticOperator, MutationOp};
use genevo::prelude::Rng;
use genevo::random::SampleUniform;

pub trait IndexedSizedContainer<T> {
    fn insert(&mut self, index: usize, value: T);
    fn remove(&mut self, index: usize);
    fn replace(&mut self, index: usize, value: T);
    fn len(&self) -> usize;
}

/// Mutates the length a container (in most cases a vector) as well as the values inside it
/// Will insert, remove, and replace values, weighted towards replace in most cases.
/// GrowableContainer must be implemented for the Genome type.
#[derive(Clone, Debug)]
pub struct SizeAndValueMutator<T> {
    /// Minimum value contained inside the vector
    min_value: T,
    /// Maximum value contained inside the vector
    max_value: T,
    /// Minimum length of the vector
    min_length: usize,
    /// Maximum length of the vector
    max_length: usize,
    /// Number of operations that the mutator will preform, will be at least once.
    mutation_percent: f32,
}

impl<T: Copy> SizeAndValueMutator<T> {
    pub fn new(
        min_value: T,
        max_value: T,
        min_length: usize,
        max_length: usize,
        mutation_percent: f32,
    ) -> Self {
        Self {
            min_value,
            max_value,
            min_length,
            max_length,
            mutation_percent,
        }
    }
}

impl<T: Clone> GeneticOperator for SizeAndValueMutator<T> {
    fn name() -> String {
        "craft mutator".to_string()
    }
}

impl<T: SampleUniform + PartialOrd + Copy, G: IndexedSizedContainer<T> + Genotype> MutationOp<G>
    for SizeAndValueMutator<T>
{
    fn mutate<R>(&self, mut genome: G, rng: &mut R) -> G
    where
        R: Rng + Sized,
    {
        // rand op count = how many times we will preform this given operation

        let mutation_counter =
            (genome.len() as f32 * rng.gen_range(0.0..self.mutation_percent)) as usize;
        for _ in 0..=mutation_counter {
            // 0 = remove item,
            // 1, 2, 3 = mutate items,
            // 1 = insert items
            // tbd: how could I allow the consumer to weight this?
            let mutation_op = rng.gen_range(0..=5);
            if mutation_op == 0 {
                let length = genome.len();
                if length > self.min_length {
                    genome.remove(rng.gen_range(0..length))
                }
            } else if (1..5).contains(&mutation_op) {
                // choose a random index to mutate
                let mutation_index = rng.gen_range(0..genome.len());
                // now overwrite that index with a random value
                let random_value = rng.gen_range(self.min_value..=self.max_value);
                genome.replace(mutation_index, random_value);
            } else if mutation_op == 5 {
                let length = genome.len();
                if length < self.max_length {
                    let index = rng.gen_range(0..length);
                    let value = rng.gen_range(self.min_value..=self.max_value);
                    genome.insert(index, value);
                }
            }
        }
        genome
    }
}
