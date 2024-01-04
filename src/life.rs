use rand::Rng;
use std::fmt;
use std::fmt::Formatter;

pub struct LifeCell {
    pub alive: bool
} impl LifeCell {
    pub fn gen() -> LifeCell {
        let alive = rand::thread_rng().gen_bool(0.5);
        LifeCell { alive }
    }

    pub fn new(alive: bool) -> LifeCell {
        LifeCell { alive }
    }
}

pub struct LifeBoard {
    grid: Vec<Vec<LifeCell>>,
    width: usize,
    height: usize,
} impl LifeBoard {
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
                let neighbors = self.get_num_alive_neighbors(row_idx, col_idx);
                let old_cell = &self.grid[row_idx][col_idx];
                let new_cell = match neighbors {
                    0|1 if old_cell.alive => LifeCell{alive: false},
                    2|3 if old_cell.alive => LifeCell{alive: true},
                    4..=8 if old_cell.alive => LifeCell{alive: false},
                    3 if !old_cell.alive => LifeCell{alive: true},
                    _ => LifeCell{alive: false},
                };
                new_col.push(new_cell);
            }
            new_grid.push(new_col);
        }
        self.grid = new_grid;
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
        let mut to_string = String::new();
        for row_idx in 0..self.width() {
            for col_idx in 0..self.height() {
                let cell = self.cell_at(row_idx, col_idx);
                let alive = if cell.alive { "T" } else { "F" };
                let cell_string = &format!("({}) ", alive)[..];
                to_string.push_str(cell_string);
            }
            to_string.push('\n');
        }
        to_string.push_str("-------------------\n");
        write!(f, "{}", to_string)
    }
}