# Conway's Game of Life:
## Summary:
This crate implements a library and a game implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life) in Rust. 
The provided implementation comes in a serial (`BaseLifeBoard`) and multi-threaded variant (`ParallelLifeBoard`).

![](./docs/game_of_life_example.gif)


## Examples:
### Print Board:
```{rust}
use life::{ParallelLifeBoard, BaseLifeBoard, Cell, LifeBoard};

const WIDTH: usize = 10;
const HEIGHT: usize = 10;
const N_THREADS: u8 = 2;

fn main() {
    let mut game = ParallelLifeBoard::from_board(
        BaseLifeBoard::gen(WIDTH, HEIGHT, Cell::gen),
        N_THREADS
    );
    println!("{game}");
    game.simulate_n_steps(10);
    println!("{game}");
}
```

### Simulate & View Game Board:
Run the [main](./src/bin/main.rs) function to start a life board simulation.
Several constants have been defined at the top of the program for customization.
* Controls:
  * _Space_: Advance to next generation
  * _P_: Pause/Unpause simulation (auto-steps after `MS_TIME_STEP` milliseconds)

## Contributors:
* Jonah Kim
