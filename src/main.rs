mod life;
use crate::life::LifeBoard;


fn main() {
    let mut life_grid = LifeBoard::gen(5, 5);
    println!("{}\n--------------------------\n", life_grid);
    life_grid.simulate();
    println!("{}", life_grid);
}