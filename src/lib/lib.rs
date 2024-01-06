mod life_implementation;
mod life_interface;

pub use life_interface::{LifeBoard, LifeCell, LifeBoardError};
pub use life_implementation::{ParallelLifeBoard, BaseLifeBoard, Cell};