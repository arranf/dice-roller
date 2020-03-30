use crate::error::DiceError;
use dice_command_parser::{dice_roll::DiceRoll, parse_line};
use rand::Rng;

/// Represents a set of homogenous dice
#[derive(PartialEq, Debug)]
pub struct Dice {
    // The number of dice in the set
    pub number_of_dice_to_roll: u32,
    // How many sides each dice in the set has
    pub sides: u32,
    // An optional positive or negative modifier to be added onto any roll using this set of dice
    modifier: Option<i32>,
}

impl Dice {
    fn from_parsed_dice_roll(parsed_roll: &DiceRoll) -> Self {
        Dice {
            number_of_dice_to_roll: parsed_roll.number_of_dice_to_roll,
            sides: parsed_roll.dice_sides,
            modifier: parsed_roll.modifier,
        }
    }

    #[must_use]
    pub fn new(number_of_dice: u32, number_of_sides: u32, modifier: Option<i32>) -> Self {
        Dice {
            number_of_dice_to_roll: number_of_dice,
            sides: number_of_sides,
            modifier,
        }
    }
}

pub enum RollType {
    WithAdvantage,
    WithDisadvantage,
    Regular,
}

/// Represents the result of rolling (a set of) `Dice`.
#[derive(PartialEq, Debug)]
pub struct DiceResult {
    /// The actual results of the dice that were cast
    pub dice_results: Vec<u32>,
    // The (total) result
    pub final_result: i32,
}

impl DiceResult {
    fn new(results: Vec<u32>, final_result: i32) -> Self {
        DiceResult {
            dice_results: results,
            final_result,
        }
    }
}

impl Dice {
    /// Creates dice from an input string
    ///
    /// # Examples
    /// ```
    /// use dice_roller::dice::Dice;
    /// let dice = Dice::create_dice("3d6 + 1").unwrap();
    /// ```
    /// # Errors
    /// Errors can occur if the dice input string is in the wrong format `DiceError::ParseError`.
    pub fn create_dice(input: &str) -> Result<Dice, DiceError> {
        Ok(Dice::from_parsed_dice_roll(&parse_line(&input)?))
    }

    /// Rolls a dice and produces a `DiceResult`. Using underlying OS RNG for the dice roll.
    #[must_use]
    pub fn roll_dice(&self) -> DiceResult {
        let mut rng = rand::thread_rng();
        self.roll_dice_from_rng(&mut rng)
    }

    /// Rolls a dice and produces a `DiceResult`. Uses a source of RNG passed in. Useful for testing.
    #[allow(clippy::cast_possible_wrap)]
    pub fn roll_dice_from_rng<R: Rng + Sized>(&self, mut rng: R) -> DiceResult {
        let mut roll_results: Vec<u32> = Vec::new();
        for _ in 0..self.number_of_dice_to_roll {
            roll_results.push(rng.gen_range(1, &self.sides));
        }
        // Wrapping is unlikely unless a huge (d2^32) dice is used or a huge (d^32) number of dice are used.
        let result = roll_results.iter().sum::<u32>() as i32 + self.modifier.unwrap_or(0);
        DiceResult::new(roll_results, result)
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
        let dice = Dice::new(1, 6, None);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, vec![2]);
    }

    #[test]
    fn modifier_added_to_predictable_result_one_d6_equals_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let modifier = Some(4);
        let dice = Dice::new(1, 6, modifier);
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2 + modifier.unwrap();
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, vec![2]);
    }

    #[test]
    fn produces_predictable_results_one_d6_parsed() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::create_dice("1d6").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 2;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, vec![2]);
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::new(3, 6, Some(2));
        let result = dice.roll_dice_from_rng(rng);
        let expected = 11;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, vec![2, 3, 4]);
    }

    #[test]
    fn produces_predictable_results_three_d6_plus_two_parsed() {
        let rng = rand_pcg::Pcg64Mcg::seed_from_u64(SEED);
        let dice = Dice::create_dice("3d6+2").expect("No error parsing dice");
        let result = dice.roll_dice_from_rng(rng);
        let expected = 11;
        assert_eq!(result.final_result, expected);
        assert_eq!(result.dice_results, vec![2, 3, 4]);
    }

    #[test]
    fn roll_dice_within_range_simple() {
        let dice = Dice::create_dice("d20").expect("No error parsing dice");
        let expected_min = 1;
        let expected_max = 20;

        for _ in 0..100_000 {
            let result = dice.roll_dice();
            if result.final_result > expected_max || result.final_result < expected_min {
                panic!("Value outside expected range");
            }
        }
    }
}
