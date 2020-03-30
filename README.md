# dice_roller
A simple Rust library for taking a dice string as an input.

```
use dice_roller::dice::Dice;

let dice = Dice::create_dice("2d20 + 1").unwrap();
let result = dice.roll_dice();
```