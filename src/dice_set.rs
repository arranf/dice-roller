use crate::{
    dice::{Dice, Operation},
    dice_result::{DiceSetResults, RollResult},
    error::DiceError,
};

use std::str::FromStr;

use dice_command_parser::parse_line;
use rand::Rng;

#[derive(Debug)]
pub struct DiceSet {
    dice: Vec<Dice>,
}

impl DiceSet {
    pub fn new(dice: Vec<Dice>) -> Self {
        DiceSet { dice }
    }

    /// Rolls a dice and produces a `DiceResult`. Using underlying OS RNG for the dice roll.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use dnd_dice_roller::dice_set::DiceSet;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice_set = DiceSet::from_str("2d20 + 1")?;
    /// let result = dice_set.roll_dice_set();
    /// # Ok::<(), DiceError>(())
    /// ```
    #[must_use]
    pub fn roll_dice_set(&self) -> DiceSetResults {
        let mut rng = rand::thread_rng();
        self.roll_dice_set_from_rng(&mut rng)
    }

    /// Rolls a dice and produces a `DiceResult`. Uses a source of RNG passed in. Useful for testing.
    ///
    /// # Examples
    /// ```
    /// use rand::SeedableRng;
    /// use dnd_dice_roller::dice_set::DiceSet;
    /// use std::str::FromStr;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
    /// let dice_set = DiceSet::from_str("3d6 + 1").unwrap();
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

impl FromStr for DiceSet {
    type Err = DiceError;
    /// Creates dice from an input string
    ///
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice_set::DiceSet;
    /// use std::str::FromStr;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice_set = DiceSet::from_str("3d6 + 1").unwrap();
    ///
    /// # Ok::<(), DiceError>(())
    /// ```
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use dnd_dice_roller::dice_set::DiceSet;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice_set = DiceSet::from_str("d6")?;
    ///
    /// # Ok::<(), DiceError>(())
    /// ```
    ///
    /// # Errors
    /// Errors can occur if the dice input string is in the wrong format `DiceError::ParseError`.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let dice_set = parse_line(&input)?
            .iter()
            .map(|d| Dice::from_parsed_dice_roll(d))
            .collect();
        Ok(Self::new(dice_set))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rand::SeedableRng;

    const SEED: u64 = 42;

    #[test]
    fn produces_predictable_results_one_d6_parsed_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = DiceSet::from_str("1d6").expect("No error parsing dice");
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], None, 2)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 2);
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_advantage_equals_three() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = DiceSet::from_str("1d6 advantage").expect("No error parsing dice");
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], Some(vec![6]), 6)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 6);
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_disadvantage_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = DiceSet::from_str("1d6 d").expect("No error parsing dice");
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2], Some(vec![6]), 2)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 2);
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two_parsed() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = DiceSet::from_str("3d6+2").expect("No error parsing dice");
        let result = dice.roll_dice_set_from_rng(rng);
        let expected = vec![RollResult::new(vec![2, 6, 5], None, 15)];
        assert_eq!(result.dice_results, expected);
        assert_eq!(result.final_result, 15);
    }

    #[test]
    fn produces_predictable_results_dice_addition() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = DiceSet::from_str("2d6+2 + d4").expect("No error parsing dice");
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
        let dice = DiceSet::from_str("2d6+2 - d4").expect("No error parsing dice");
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
        let dice = DiceSet::from_str("2d6+2 + d10+2 - 2d4-1").expect("No error parsing dice");
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
