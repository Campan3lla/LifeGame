mod life;
use crate::life::LifeGrid;


fn main() {
    let mut life_grid = LifeGrid::new(5, 5);
    println!("{}", life_grid);
    life_grid.simulate();
    println!("{}", life_grid);
}