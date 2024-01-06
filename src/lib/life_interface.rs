use std::fmt::{Debug};

pub trait LifeBoard<T: LifeCell<T>>: PartialEq + Clone {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn simulate(&mut self);
    fn simulate_n_steps(&mut self, n: usize);
    fn next_cell_state_at(&self, x:usize, y:usize) -> Option<T>;
    fn cell_at(&self, x:usize, y:usize) -> Option<T>;
    fn num_alive_neighbors_at(&self, x: usize, y: usize) -> u8;
    fn is_cell_alive(&self, x: usize, y: usize) -> Option<bool>;
    fn to_vec_matrix(&self) -> Vec<Vec<T>>;
}


pub trait LifeCell<T: LifeCell<T>>: PartialEq + Clone {
    fn is_alive(&self) -> bool;
    fn to_alive(&self) -> T;
    fn to_dead(&self) -> T;
}

#[derive(Debug)]
pub enum LifeBoardError {
    InvalidBoard(String),
    InvalidIndex(String),
}
