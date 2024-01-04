use rand::Rng;
use std::fmt;
use std::fmt::Formatter;

pub struct LifeCell {
    alive: bool
} impl LifeCell {
    pub fn gen() -> LifeCell {
        let alive = rand::thread_rng().gen_bool(0.5);
        LifeCell { alive }
    }

    pub fn new(alive: bool) -> LifeCell {
        LifeCell { alive }
    }
}

pub enum LifeError {
    InvalidBoard(String),
}



pub struct LifeBoard {
    grid: Vec<Vec<LifeCell>>,
    width: usize,
    height: usize,
} impl LifeBoard {
    pub fn new(grid: Vec<Vec<LifeCell>>) -> Result<LifeBoard, LifeError> {
        let width = match grid.len() {
            0 => return Err(
                LifeError::InvalidBoard(String::from("Board must be at least one cell wide."))
            ),
            len => len,
        };
        let height = match grid[0].len() {
            0 => return Err(
                LifeError::InvalidBoard(String::from("Board must be at least one cell tall."))
            ),
            len => len,
        };
        for col in &grid {
            if col.len() != height {
                return Err(
                    LifeError::InvalidBoard(String::from("Board must have columns of consistent size."))
                )
            }
        }
        return Ok(LifeBoard { grid, width, height })
    }

    pub fn gen(width: usize, height: usize) -> LifeBoard {
        let mut grid: Vec<Vec<LifeCell>> = Vec::with_capacity(width);
        for _ in 0..width {
            let mut col = Vec::with_capacity(height);
            for _ in 0..height {
                col.push(LifeCell::gen());
            }
            grid.push(col);
        }

        LifeBoard { grid, width, height }
    }

    pub fn simulate(&mut self) {
        let mut new_grid: Vec<Vec<LifeCell>> = Vec::with_capacity(self.width);
        for row_idx in 0..self.width {
            let mut new_col = Vec::with_capacity(self.height);
            for col_idx in 0..self.height {
                let new_cell = self.next_cell_state(row_idx, col_idx);
                new_col.push(new_cell);
            }
            new_grid.push(new_col);
        }
        self.grid = new_grid;
    }

    fn next_cell_state(&self, x:usize, y:usize) -> LifeCell {
        let neighbors = self.get_num_alive_neighbors(x, y);
        let old_cell = &self.grid[x][y];
        return match neighbors {
            0|1 if old_cell.alive => LifeCell{alive: false},
            2|3 if old_cell.alive => LifeCell{alive: true},
            4..=8 if old_cell.alive => LifeCell{alive: false},
            3 if !old_cell.alive => LifeCell{alive: true},
            _ => LifeCell{alive: false},
        };
    }

    pub fn get_num_alive_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut neighbors = 0u8;
        for dx in 0..3 {
            for dy in 0..3 {
                if dx == 1 && dy == 1 {
                    continue
                } else {
                    let (x_test, y_test) = ((x as i64 - 1) + dx, (y as i64 - 1) + dy);
                    if let Some(is_alive) = self.is_cell_alive(x_test, y_test) {
                        if is_alive { neighbors += 1; }
                    }
                }
            }
        }
        neighbors
    }

    pub fn is_cell_alive(&self, x: i64, y: i64) -> Option<bool> {
        let (x, y) = match (x, y) {
            (x, _) if x < 0 => return None,
            (_, y) if y < 0 => return None,
            _ => (x as usize, y as usize),
        };
        match self.grid.get(x) {
            Some(row) => match row.get(y) {
                Some(cell) => Some(cell.alive),
                None => None
            },
            None => None
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }
    pub fn cell_at(&self, x: usize, y: usize) -> &LifeCell { &self.grid[x][y] }
} impl fmt::Display for LifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row_idx in 0..self.width() {
            for col_idx in 0..self.height() {
                let cell = self.cell_at(row_idx, col_idx);
                let alive = if cell.alive { "T" } else { "F" };
                let cell_string = format!("{} ", alive);
                write!(f, "{}", cell_string)?;
            }
            let newline = if row_idx == self.width-1 { "" } else { "\n" };
            write!(f, "{}", newline)?;
        }
        write!(f, "{}", "")
    }
}

#[cfg(test)]
mod tests {
    use crate::life::{LifeBoard, LifeCell, LifeError};

    fn assert_contains(actual: String, expected: &str) {
        assert!(
            actual.contains(expected),
            "Expected \"{actual}\" to contain \"{expected}\"")
    }

    #[test]
    fn test_exception_life_board_new_invalid_row() {
        let grid: Vec<Vec<LifeCell>> = Vec::new();
        match LifeBoard::new(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeError::InvalidBoard(error)) => {
                assert_contains(error, "at least one cell wide");
            }
        };
    }
}