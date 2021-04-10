use crate::dice_result::RollResult;

use dice_command_parser::{
    dice_roll::Operation as CommandOperation, dice_roll::RollType as CommandRollType,
    dice_roll_with_op::DiceRollWithOp,
};
use rand::Rng;

use std::cmp::{max, min};

/// Represents a set of homogenous dice. E.G. Three d6
#[derive(PartialEq, Debug)]
pub struct Dice {
    /// The number of dice in the set of homegenous dice.
    pub number_of_dice_to_roll: u32,
    /// How many sides each dice in the set has.
    pub sides: u32,
    /// An optional positive or negative modifier to be added onto any roll using this set of dice.
    pub modifier: Option<i32>,
    /// Whether the dice set should roll as a `RollType::Regular` (which rolls once), or as a `RollType::Advantage` or `RollType::Disadvantage` which rolls twice.
    pub roll_type: RollType,
    /// Whether this dice should be added or taken from the overall total
    pub operation: Operation,
}

/// Represents the advantage or disadvantage on a roll.
#[derive(PartialEq, Debug)]
pub enum RollType {
    /// The roll occurs twice, with the highest result being taken.
    Advantage,
    /// The roll occurs twice, with the lowest result being taken.
    Disadvantage,
    /// The roll occurs once and the result is taken.
    Regular,
}

/// Represents whether the dice result should be added or taken away from the total.
#[derive(PartialEq, Debug)]
pub enum Operation {
    /// The roll should be taken away from the overall total
    Addition,
    /// The roll should be added to the overall total
    Subtraction,
}

impl Dice {
    pub(crate) fn from_parsed_dice_roll(parsed_roll: &DiceRollWithOp) -> Self {
        let roll_type = match parsed_roll.dice_roll.roll_type {
            CommandRollType::Regular => RollType::Regular,
            CommandRollType::WithAdvantage => RollType::Advantage,
            CommandRollType::WithDisadvantage => RollType::Disadvantage,
        };

        let operation = match parsed_roll.operation {
            CommandOperation::Addition => Operation::Addition,
            CommandOperation::Subtraction => Operation::Subtraction,
        };

        Dice {
            number_of_dice_to_roll: parsed_roll.dice_roll.number_of_dice_to_roll,
            sides: parsed_roll.dice_roll.dice_sides,
            modifier: parsed_roll.dice_roll.modifier,
            roll_type,
            operation,
        }
    }

    /// Constructs a new dice
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice::{Dice, RollType, Operation};
    /// // A single d20 with a plus five modifier and advantage
    /// let dice = Dice::new(1, 20, Some(5), RollType::Advantage, Operation::Addition);
    /// ```
    #[must_use]
    pub fn new(
        number_of_dice: u32,
        number_of_sides: u32,
        modifier: Option<i32>,
        roll_type: RollType,
        operation: Operation,
    ) -> Self {
        Dice {
            number_of_dice_to_roll: number_of_dice,
            sides: number_of_sides,
            modifier,
            roll_type,
            operation,
        }
    }

    /// Rolls a dice and produces a `RollResult`. Using underlying OS RNG for the dice roll.
    ///
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice::{Dice, RollType, Operation};
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice = Dice::new(1, 10, None, RollType::Regular, Operation::Addition);
    /// let result = dice.roll_dice();
    /// # Ok::<(), DiceError>(())
    /// ```
    #[must_use]
    pub fn roll_dice(&self) -> RollResult {
        let mut rng = rand::thread_rng();
        self.roll_dice_from_rng(&mut rng)
    }

    /// Rolls a dice and produces a `RollResult`. Uses a source of RNG passed in. Useful for testing.
    ///
    /// # Examples
    /// ```
    /// use rand::SeedableRng;
    /// use dnd_dice_roller::dice::{Dice, RollType, Operation};
    ///
    /// let rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
    /// let dice = Dice::new(1, 6, None, RollType::Regular, Operation::Addition);
    /// let result = dice.roll_dice_from_rng(rng);
    /// assert_eq!(result.result, 2);
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn roll_dice_from_rng<R: Rng + Sized>(&self, mut rng: R) -> RollResult {
        let current_roll_set_size = self.number_of_dice_to_roll as usize;
        let mut first_roll_results: Vec<u32> = Vec::with_capacity(current_roll_set_size);
        for _ in 0..self.number_of_dice_to_roll {
            first_roll_results.push(rng.gen_range(1..=self.sides));
        }

        let second_roll_results: Option<Vec<u32>> = match self.roll_type {
            RollType::Advantage | RollType::Disadvantage => {
                let mut second_roll_results: Vec<u32> = Vec::with_capacity(current_roll_set_size);
                for _ in 0..self.number_of_dice_to_roll {
                    second_roll_results.push(rng.gen_range(1..=self.sides));
                }
                Some(second_roll_results)
            }
            RollType::Regular => None,
        };
        // Wrapping is unlikely unless a huge (d2^32) dice is used or a huge (d^32) number of dice are used.
        let result = match self.roll_type {
            RollType::Regular => {
                first_roll_results.iter().sum::<u32>() as i32 + self.modifier.unwrap_or(0)
            }
            RollType::Advantage => {
                let modifier = self.modifier.unwrap_or(0);
                let first_result = first_roll_results.iter().sum::<u32>() as i32;
                let second_result = second_roll_results
                    .as_ref()
                    .expect("Expect advantage roll to have second roll results")
                    .iter()
                    .sum::<u32>() as i32;
                max(first_result + modifier, second_result + modifier)
            }
            RollType::Disadvantage => {
                let modifier = self.modifier.unwrap_or(0);
                let first_result = first_roll_results.iter().sum::<u32>() as i32;
                let second_result = second_roll_results
                    .as_ref()
                    .expect("Expect disadvantage roll to have second roll results")
                    .iter()
                    .sum::<u32>() as i32;
                min(first_result + modifier, second_result + modifier)
            }
        };

        RollResult::new(first_roll_results, second_roll_results, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    const SEED: u64 = 42;

    #[test]
    fn produces_predictable_results_one_d6_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Regular, Operation::Addition);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.result, expected);
        assert_eq!(result.first_roll, vec![2]);
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_equals_two_plus_modifier() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let modifier = Some(4);
        let dice = Dice::new(1, 6, modifier, RollType::Regular, Operation::Addition);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2 + modifier.unwrap();
        assert_eq!(result.result, expected);
        assert_eq!(result.first_roll, vec![2]);
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_with_advantage_equals_six() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Advantage, Operation::Addition);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 6;
        assert_eq!(result.result, expected);
        assert_eq!(result.first_roll, vec![2]);
        assert_eq!(result.second_roll, Some(vec![6]))
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_with_disadvantage_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Disadvantage, Operation::Addition);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.result, expected);
        assert_eq!(result.first_roll, vec![2]);
        assert_eq!(result.second_roll, Some(vec![6]))
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(3, 6, Some(2), RollType::Regular, Operation::Addition);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 15;
        assert_eq!(result.result, expected);
        assert_eq!(result.first_roll, vec![2, 6, 5]);
    }

    #[test]
    fn roll_dice_within_range_simple() {
        let dice = Dice::new(1, 20, None, RollType::Regular, Operation::Addition);
        let expected_min = 1;
        let expected_max = 20;

        for _ in 0..100_000 {
            let result = dice.roll_dice();
            let result = result.result;
            if result > expected_max || result < expected_min {
                panic!("Value outside expected range");
            }
        }
    }

    #[test]
    fn roll_dice_within_range_check_occurences() {
        let dice = Dice::new(1, 20, None, RollType::Regular, Operation::Addition);
        let expected_min = 1;
        let expected_max = 20;

        let number_of_rolls = 100_000;
        let mut results: Vec<i32> = Vec::with_capacity(100_000);
        for _ in 0..number_of_rolls {
            let roll_result = dice.roll_dice();

            if roll_result.result > expected_max || roll_result.result < expected_min {
                panic!("Value outside expected range");
            }
            results.push(roll_result.result);
        }

        let mut results = results.iter();

        for searching_for in expected_min..=expected_max {
            if !results.any(|&item| item == searching_for) {
                panic!("Could not find value expected value in all iterations of results");
            }
        }
    }
}
