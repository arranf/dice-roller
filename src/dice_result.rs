use std::fmt;

/// Represents the result of rolling (a set of) `Dice`.
#[derive(PartialEq, Debug)]
pub struct DiceSetResults {
    /// The actual results of the dice that were cast
    pub dice_results: Vec<RollResult>,
    /// The (total) result
    pub final_result: i32,
}

impl DiceSetResults {
    pub(crate) fn new(results: Vec<RollResult>, final_result: i32) -> Self {
        DiceSetResults {
            dice_results: results,
            final_result,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct RollResult {
    /// Present on `RollType::Advantage`, `RollType::Disadvantage` and `RollType::Regular` rolls.
    pub first_roll: Vec<u32>,
    /// Only present on `RollType::Advantage`, `RollType::Disadvantage` rolls.
    pub second_roll: Option<Vec<u32>>,
    pub result: i32,
}

impl RollResult {
    pub(crate) fn new(first_roll: Vec<u32>, second_roll: Option<Vec<u32>>, result: i32) -> Self {
        RollResult {
            first_roll,
            second_roll,
            result,
        }
    }
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.second_roll {
            None => write!(f, "{:?}", self.first_roll),
            Some(second_roll) => write!(f, "[{:?}, {:?}]", self.first_roll, second_roll),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn format_raw_result_with_only_one_roll() {
        let raw_result = RollResult::new(vec![1, 2, 3, 4], None, 7);
        assert_eq!("[1, 2, 3, 4]", format!("{}", raw_result));
    }

    #[test]
    fn format_raw_result_with_two_rolls() {
        let raw_result = RollResult::new(vec![4, 2, 1, 3], Some(vec![5, 2, 3, 4]), 14);
        assert_eq!("[[4, 2, 1, 3], [5, 2, 3, 4]]", format!("{}", raw_result));
    }
}
