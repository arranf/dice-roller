use crate::error::DiceError;
use dice_command_parser::{
    dice_roll::DiceRoll, dice_roll::RollType as CommandRollType, parse_line,
};
use rand::Rng;

use std::cmp::{max, min};
use std::fmt;
use std::str::FromStr;

/// Represents a set of homogenous dice
#[derive(PartialEq, Debug)]
pub struct Dice {
    /// The number of dice in the set.
    pub number_of_dice_to_roll: u32,
    /// How many sides each dice in the set has.
    pub sides: u32,
    /// An optional positive or negative modifier to be added onto any roll using this set of dice.
    pub modifier: Option<i32>,
    /// Whether the dice set should roll as a `RollType::Regular` (which rolls once), or as a `RollType::Advantage` or `RollType::Disadvantage` which rolls twice.
    pub roll_type: RollType,
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

/// Represents the result of rolling (a set of) `Dice`.
#[derive(PartialEq, Debug)]
pub struct DiceResult {
    /// The actual results of the dice that were cast
    pub dice_results: RawResults,
    /// The (total) result
    pub final_result: i32,
}

impl DiceResult {
    fn new(results: RawResults, final_result: i32) -> Self {
        DiceResult {
            dice_results: results,
            final_result,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct RawResults {
    /// Present on `RollType::Advantage`, `RollType::Disadvantage`, and `RollType::Regular` rolls.
    pub first_roll: Vec<u32>,
    /// Present on `RollType::Advantage`, `RollType::Disadvantage` rolls.
    pub second_roll: Option<Vec<u32>>,
}

impl RawResults {
    fn new(first_roll: Vec<u32>, second_roll: Option<Vec<u32>>) -> Self {
        RawResults {
            first_roll,
            second_roll,
        }
    }
}

impl fmt::Display for RawResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.second_roll {
            None => write!(f, "{:?}", self.first_roll),
            Some(second_roll) => write!(f, "[{:?}, {:?}]", self.first_roll, second_roll),
        }
    }
}

impl FromStr for Dice {
    type Err = DiceError;
    /// Creates dice from an input string
    ///
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice::Dice;
    /// use std::str::FromStr;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice = Dice::from_str("3d6 + 1").unwrap();
    ///
    /// # Ok::<(), DiceError>(())
    /// ```
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use dnd_dice_roller::dice::Dice;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice = Dice::from_str("d6")?;
    ///
    /// # Ok::<(), DiceError>(())
    /// ```
    ///
    /// # Errors
    /// Errors can occur if the dice input string is in the wrong format `DiceError::ParseError`.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Dice::from_parsed_dice_roll(&parse_line(&input)?))
    }
}

impl Dice {
    fn from_parsed_dice_roll(parsed_roll: &DiceRoll) -> Self {
        let roll_type = match parsed_roll.roll_type {
            CommandRollType::Regular => RollType::Regular,
            CommandRollType::WithAdvantage => RollType::Advantage,
            CommandRollType::WithDisadvantage => RollType::Disadvantage,
        };

        Dice {
            number_of_dice_to_roll: parsed_roll.number_of_dice_to_roll,
            sides: parsed_roll.dice_sides,
            modifier: parsed_roll.modifier,
            roll_type,
        }
    }

    /// Constructs a new dice
    /// # Examples
    /// ```
    /// use dnd_dice_roller::dice::{Dice, RollType};
    /// // A single d20 with a plus five modifier and advantage
    /// let dice = Dice::new(1, 20, Some(5), RollType::Advantage);
    /// ```
    #[must_use]
    pub fn new(
        number_of_dice: u32,
        number_of_sides: u32,
        modifier: Option<i32>,
        roll_type: RollType,
    ) -> Self {
        Dice {
            number_of_dice_to_roll: number_of_dice,
            sides: number_of_sides,
            modifier,
            roll_type,
        }
    }

    /// Rolls a dice and produces a `DiceResult`. Using underlying OS RNG for the dice roll.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use dnd_dice_roller::dice::Dice;
    /// # use dnd_dice_roller::error::DiceError;
    ///
    /// let dice = Dice::from_str("2d20 + 1")?;
    /// let result = dice.roll_dice();
    /// # Ok::<(), DiceError>(())
    /// ```
    #[must_use]
    pub fn roll_dice(&self) -> DiceResult {
        let mut rng = rand::thread_rng();
        self.roll_dice_from_rng(&mut rng)
    }

    /// Rolls a dice and produces a `DiceResult`. Uses a source of RNG passed in. Useful for testing.
    ///
    /// # Examples
    /// ```
    /// use rand::SeedableRng;
    /// use dnd_dice_roller::dice::{Dice, RollType};
    ///
    /// let rng = rand_pcg::Pcg64Mcg::seed_from_u64(42);
    /// let dice = Dice::new(1, 6, None, RollType::Regular);
    /// let result = dice.roll_dice_from_rng(rng);
    /// assert_eq!(result.final_result, 2);
    /// ```
    #[allow(clippy::cast_possible_wrap)]
    pub fn roll_dice_from_rng<R: Rng + Sized>(&self, mut rng: R) -> DiceResult {
        let current_roll_set_size = self.number_of_dice_to_roll as usize;
        let mut first_roll_results: Vec<u32> = Vec::with_capacity(current_roll_set_size);
        for _ in 0..self.number_of_dice_to_roll {
            first_roll_results.push(rng.gen_range(1, &self.sides + 1));
        }

        let second_roll_results: Option<Vec<u32>> = match self.roll_type {
            RollType::Advantage | RollType::Disadvantage => {
                let mut second_roll_results: Vec<u32> = Vec::with_capacity(current_roll_set_size);
                for _ in 0..self.number_of_dice_to_roll {
                    second_roll_results.push(rng.gen_range(1, &self.sides + 1));
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

        let raw_results = RawResults::new(first_roll_results, second_roll_results);
        DiceResult::new(raw_results, result)
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
        let dice = Dice::new(1, 6, None, RollType::Regular);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], None));
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_equals_two_plus_modifier() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let modifier = Some(4);
        let dice = Dice::new(1, 6, modifier, RollType::Regular);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2 + modifier.unwrap();
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], None));
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::from_str("1d6").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], None));
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_with_advantage_equals_six() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Advantage);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 6;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], Some(vec![6])));
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_advantage_equals_three() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::from_str("1d6 advantage").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 6;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], Some(vec![6])));
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_with_disadvantage_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(1, 6, None, RollType::Disadvantage);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], Some(vec![6])));
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed_with_disadvantage_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::from_str("1d6 d").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2], Some(vec![6])));
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(3, 6, Some(2), RollType::Regular);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 15;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2, 6, 5], None));
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two_parsed() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::from_str("3d6+2").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 15;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, RawResults::new(vec![2, 6, 5], None));
    }

    #[test]
    fn roll_dice_within_range_simple() {
        let dice = Dice::from_str("d20").expect("No error parsing dice");
        let expected_min = 1;
        let expected_max = 20;

        for _ in 0..100_000 {
            let result = dice.roll_dice();
            if result.final_result > expected_max || result.final_result < expected_min {
                panic!("Value outside expected range");
            }
        }
    }
    #[test]
    fn roll_dice_within_range_check_occurences() {
        let dice = Dice::new(1, 20, None, RollType::Regular);
        let expected_min = 1;
        let expected_max = 20;

        let number_of_rolls = 100_000;
        let mut results: Vec<i32> = Vec::with_capacity(100_000);
        for _ in 0..number_of_rolls {
            let result = dice.roll_dice();
            if result.final_result > expected_max || result.final_result < expected_min {
                panic!("Value outside expected range");
            }
            results.push(result.final_result);
        }

        let mut results = results.iter();

        for searching_for in expected_min..=expected_max {
            if !results.any(|&item| item == searching_for) {
                panic!(format!(
                    "Could not find value {} in {} iterations of results",
                    searching_for, number_of_rolls
                ));
            }
        }
    }

    #[test]
    fn format_raw_result_with_only_one_roll() {
        let raw_result = RawResults::new(vec![1, 2, 3, 4], None);
        assert_eq!("[1, 2, 3, 4]", format!("{}", raw_result));
    }

    #[test]
    fn format_raw_result_with_two_rolls() {
        let raw_result = RawResults::new(vec![4, 2, 1, 3], Some(vec![5, 2, 3, 4]));
        assert_eq!("[[4, 2, 1, 3], [5, 2, 3, 4]]", format!("{}", raw_result));
    }
}
