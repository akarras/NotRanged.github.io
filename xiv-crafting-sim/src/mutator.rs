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

    /// Sets the minimum value that will be used to mutate the genome
    pub fn set_min_value(&mut self, value: T) -> &mut Self {
        self.min_value = value;
        self
    }

    /// Gets the minimum value to generate values
    pub fn get_min_value(&self) -> T {
        self.min_value
    }

    /// Sets the maximum value that will be used to mutate the genome
    pub fn set_max_value(&mut self, value: T) -> &mut Self {
        self.max_value = value;
        self
    }

    /// Gets the maximum value used to mutate the genome
    pub fn get_max_value(&self) -> T {
        self.max_value
    }

    /// Sets the maximum length that the container can grow to
    pub fn set_max_length(&mut self, length: usize) -> &mut Self {
        self.max_length = length;
        self
    }

    /// Gets the maximum length that the container can grow to
    pub fn get_max_length(&self) -> usize {
        self.max_length
    }

    /// Sets the minimum length that the container must be
    pub fn set_min_length(&mut self, length: usize) -> &mut Self {
        self.min_length = length;
        self
    }

    /// Gets the minimum length that the container must be
    pub fn get_min_length(&self) -> usize {
        self.min_length
    }

    /// Sets the percentage of mutation to occur, ranges from 0 to 1
    pub fn set_mutation_percentage(&mut self, value: f32) -> &mut Self {
        self.mutation_percent = value;
        self
    }

    /// Gets the percentage of mutation to occur, ranges from 0 to 1
    pub fn get_mutation_percentage(&self) -> f32 {
        self.mutation_percent
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
