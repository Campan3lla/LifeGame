use std::fmt::{Debug, Display};

pub trait LifeBoard<T: LifeCell + PartialEq + Clone>: PartialEq + Clone {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn simulate(&mut self);
    fn simulate_n_steps(&mut self, n: usize);
    fn next_cell_state_at(&self, x:usize, y:usize) -> Result<&T, LifeBoardError>;
    fn cell_at(&self, x:usize, y:usize) -> Result<&T, LifeBoardError>;
    fn num_alive_neighbors_at(&self, x: usize, y: usize) -> Result<u8, LifeBoardError>;
    fn is_cell_alive(&self, x: usize, y: usize) -> Result<bool, LifeBoardError>;
    fn to_vec_matrix(&self) -> Vec<Vec<T>>;
}


pub trait LifeCell {
    fn is_alive(&self) -> bool;
}

#[derive(Debug)]
pub enum LifeBoardError {
    InvalidBoard(String),
    InvalidIndex(String),
}
