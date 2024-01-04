mod life;
use crate::life::LifeBoard;


fn main() {
    let mut life_grid = LifeBoard::gen(5, 5);
    println!("{}", life_grid);
    println!("---------------");
    life_grid.simulate();
    println!("{}", life_grid);
}