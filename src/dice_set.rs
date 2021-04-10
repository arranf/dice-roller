use crate::{
    dice::{Dice, Operation},
    dice_result::{DiceSetResults, RollResult},
};

use rand::Rng;

#[derive(Debug)]
/// Represents a set of non-homogenous dice e.g. d20+2 + d4.
pub struct DiceSet {
    dice: Vec<Dice>,
}

impl DiceSet {
    /// Create a new dice set
    pub fn new(dice: Vec<Dice>) -> Self {
        DiceSet { dice }
    }

    /// Rolls a set of dice and produces a `DiceSetResults`. Using underlying OS RNG for the dice roll.
    ///
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice::{Dice, RollType, Operation};
    /// use dnd_dice_roller::dice_set::DiceSet;
    ///
    /// let dice = vec![Dice::new(2, 20, Some(1), RollType::Regular, Operation::Addition)];
    /// let dice_set = DiceSet::new(dice);
    /// let result = dice_set.roll_dice_set();
    /// ```
    #[must_use]
    pub fn roll_dice_set(&self) -> DiceSetResults {
        let mut rng = rand::thread_rng();
        self.roll_dice_set_from_rng(&mut rng)
    }

    /// Rolls a set of dice and produces a `DiceSetResults`. Uses a source of RNG passed in. Useful for testing.
    ///
    /// # Examples
    /// ```
    /// use rand::SeedableRng;
    /// use dnd_dice_roller::dice::{Dice, RollType, Operation};
    /// use dnd_dice_roller::dice_set::DiceSet;
    ///
    /// let rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
    /// let dice = vec![Dice::new(3, 6, Some(1), RollType::Regular, Operation::Addition)];
    /// let dice_set = DiceSet::new(dice);
    /// let result = dice_set.roll_dice_set_from_rng(rng);
    /// assert_eq!(result.final_result, 14);
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn roll_dice_set_from_rng<R: Rng + Sized>(&self, mut rng: R) -> DiceSetResults {
        let results: Vec<RollResult> = self
            .dice
            .iter()
            .map(|d| d.roll_dice_from_rng(&mut rng))
            .collect();
        let total = results.iter().enumerate().fold(0, |acc, (index, roll)| {
            match self.dice.get(index).unwrap().operation {
                Operation::Addition => acc + roll.result,
                Operation::Subtraction => acc - roll.result,
            }
        });

        DiceSetResults::new(results, total)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::dice::{Dice, Operation, RollType};

    use rand::SeedableRng;

    const SEED: u64 = 42;

    #[test]
    fn produces_predictable_results_one_d6_parsed_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Regular, Operation::Addition);
        let dice = DiceSet::new(vec![dice]);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], None, 2)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 2);
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_advantage_equals_three() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);

        let dice = Dice::new(1, 6, None, RollType::Advantage, Operation::Addition);
        let dice = DiceSet::new(vec![dice]);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], Some(vec![6]), 6)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 6);
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_disadvantage_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Disadvantage, Operation::Addition);
        let dice = DiceSet::new(vec![dice]);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], Some(vec![6]), 2)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 2);
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two_parsed() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(3, 6, Some(2), RollType::Regular, Operation::Addition);
        let dice = DiceSet::new(vec![dice]);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2, 6, 5], None, 15)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 15);
    }

    #[test]
    fn produces_predictable_results_dice_addition() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);

        let dice = vec![
            Dice::new(2, 6, Some(2), RollType::Regular, Operation::Addition),
            Dice::new(1, 4, None, RollType::Regular, Operation::Addition),
        ];

        let dice = DiceSet::new(dice);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![
            RollResult::new(vec![2, 6], None, 10),
            RollResult::new(vec![4], None, 4),
        ];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 14);
    }

    #[test]
    fn produces_predictable_results_dice_subtraction() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);

        let dice = vec![
            Dice::new(2, 6, Some(2), RollType::Regular, Operation::Addition),
            Dice::new(1, 4, None, RollType::Regular, Operation::Subtraction),
        ];

        let dice = DiceSet::new(dice);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![
            RollResult::new(vec![2, 6], None, 10),
            RollResult::new(vec![4], None, 4),
        ];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 6);
    }

    #[test]
    fn produces_predictable_results_dice_combined() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);

        let dice = vec![
            Dice::new(2, 6, Some(2), RollType::Regular, Operation::Addition),
            Dice::new(1, 10, Some(2), RollType::Regular, Operation::Addition),
            Dice::new(2, 4, Some(-1), RollType::Regular, Operation::Subtraction),
        ];

        let dice = DiceSet::new(dice);
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![
            RollResult::new(vec![2, 6], None, 10),
            RollResult::new(vec![2], None, 4),
            RollResult::new(vec![3, 3], None, 5),
        ];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 9);
    }
}
