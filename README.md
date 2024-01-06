# Conway's Game of Life:
## Summary:
This crate implements a library and a game implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway's_Game_of_Life) in Rust. 
The provided implementation comes in a serial (`BaseLifeBoard`) and multi-threaded variant (`ParallelLifeBoard`).

<img src="./docs/game_of_life_simulation.mov" width="200px" alt="game of life">
<img src="./docs/game_of_life.mp4" width="200px" alt="game of life">
<img src="./docs/game_of_life_sim.mp4" width="200px" alt="game of life">


## Examples:
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

## Contributors:
* Jonah Kim
