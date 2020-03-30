# dice_roller
A simple Rust library for taking a DnD like dice string as an input and calculating a result.

Supports advantage and disadvantage.



## Usage

```
use dice_roller::dice::{Dice, RollType};
use std::str::FromStr;

let dice = Dice::from_str("2d20 + 1")?;
// Roll dice uses thread RNG
let result = dice.roll_dice();

A single d20 with a plus five modifier and advantage
let second_dice = Dice::new(1, 20, Some(5), RollType::Advantage);
let results = second_dice.roll_dice();
```

## Example inputs
```
d6
2d6
2d6 + 3
d20 advantage
d20 adv
d20 a
2d20 + 4 advantage
2d20 - 2 adv
1d6 - 1 disadvantage
1d6 dadv
d6 d
```