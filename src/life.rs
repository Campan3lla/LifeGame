use rand::Rng;
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::Range;
use std::sync::{Arc, mpsc};
use std::thread;
use crate::life_interface::LifeBoardError;

#[derive(Clone)]
pub struct LifeBoard {
    grid: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
} impl LifeBoard {
    pub fn new(grid: Vec<Vec<Cell>>) -> Result<LifeBoard, LifeBoardError> {
        let width = match grid.len() {
            0 => return Err(
                LifeBoardError::InvalidBoard(String::from("Board must be at least one cell wide."))
            ),
            len => len,
        };
        let height = match grid[0].len() {
            0 => return Err(
                LifeBoardError::InvalidBoard(String::from("Board must be at least one cell tall."))
            ),
            len => len,
        };
        for col in &grid {
            if col.len() != height {
                return Err(
                    LifeBoardError::InvalidBoard(String::from("Board must have columns of consistent size."))
                )
            }
        }
        return Ok(LifeBoard { grid, width, height })
    }

    pub fn gen(width: usize, height: usize) -> LifeBoard {
        let mut grid: Vec<Vec<Cell>> = Vec::with_capacity(width);
        for _ in 0..width {
            let mut col = Vec::with_capacity(height);
            for _ in 0..height {
                col.push(Cell::gen());
            }
            grid.push(col);
        }

        LifeBoard { grid, width, height }
    }

    pub fn simulate(&self) -> LifeBoard {
        let mut new_grid: Vec<Vec<Cell>> = Vec::with_capacity(self.width);
        for row_idx in 0..self.width {
            let mut new_col = Vec::with_capacity(self.height);
            for col_idx in 0..self.height {
                let new_cell = self.next_cell_state(row_idx, col_idx);
                new_col.push(new_cell);
            }
            new_grid.push(new_col);
        }
        return LifeBoard { grid: new_grid, width: self.width, height: self.height };
    }

    pub fn simulate_n_steps(&self, steps: u16) -> LifeBoard {
        if steps == 0 {
            LifeBoard { grid: self.grid.clone(), width: self.width, height: self.height }
        } else {
            let mut board = self.simulate();
            for _ in 1..steps {
                board = board.simulate();
            }
            return board;
        }
    }

    fn next_cell_state(&self, x:usize, y:usize) -> Cell {
        let neighbors = self.get_num_alive_neighbors(x, y);
        let old_cell = &self.grid[x][y];
        return match neighbors {
            0|1 if old_cell.alive => Cell {alive: false},
            2|3 if old_cell.alive => Cell {alive: true},
            4..=8 if old_cell.alive => Cell {alive: false},
            3 if !old_cell.alive => Cell {alive: true},
            _ => Cell {alive: false},
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
    pub fn cell_at(&self, x: usize, y: usize) -> &Cell { &self.grid[x][y] }

    pub fn grid_fmt(&self, f: &mut Formatter<'_>, alive_cell: &str, dead_cell: &str, dbg: bool) -> fmt::Result {
        for col_idx in 0..self.height() {
            for row_idx in 0..self.width() {
                let cell = self.cell_at(row_idx, col_idx);
                let alive = if cell.alive { alive_cell } else { dead_cell };
                let cell_string = if dbg {
                    format!("(alv={alive}, {row_idx}, {col_idx}) ")
                } else {
                    format!("{} ", alive)
                };
                write!(f, "{}", cell_string)?;
            }
            let newline = if col_idx == self.width-1 { "" } else { "\n" };
            write!(f, "{}", newline)?;
        }
        write!(f, "{}", "")
    }
} impl Display for LifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.grid_fmt(f, "*", " ", false)
    }
} impl Debug for LifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.grid_fmt(f, "T", "F", true)
    }
} impl PartialEq for LifeBoard {
    fn eq(&self, other: &Self) -> bool {
        let mut all_eq = true;
        for (expected_row, actual_row) in self.grid.iter().zip(other.grid.iter()) {
            all_eq &= expected_row.eq(actual_row);
        }
        all_eq
    }
}

pub struct ParallelLifeBoard {
    board: Arc<LifeBoard>,
    nthreads: usize,
    thread_row_ranges: Vec<Range<usize>>,
} impl ParallelLifeBoard {
    pub fn gen(width: usize, height: usize, nthreads: u8) -> ParallelLifeBoard {
        ParallelLifeBoard {
            board: Arc::new(LifeBoard::gen(width, height)),
            nthreads: nthreads as usize,
            thread_row_ranges: ParallelLifeBoard::row_ranges(width, nthreads as usize)
        }
    }

    pub fn from(board: LifeBoard, nthreads: u8) -> ParallelLifeBoard {
        ParallelLifeBoard {
            thread_row_ranges: ParallelLifeBoard::row_ranges(board.width, nthreads as usize),
            board: Arc::new(board),
            nthreads: (nthreads as usize),
        }
    }
    pub fn from_grid<A, B>(collection: A, nthreads: u8) -> Result<ParallelLifeBoard, LifeBoardError>
        where
            A: IntoIterator<Item=B>,
            B: IntoIterator<Item=bool>
    {
        let grid = collection.into_iter().map(|row| {
            row.into_iter().map(|alive| {
                Cell { alive }
            }).collect()
        }).collect();
        let board = LifeBoard::new(grid)?;
        Ok(ParallelLifeBoard {
            thread_row_ranges: ParallelLifeBoard::row_ranges(board.width, nthreads as usize),
            nthreads: nthreads as usize,
            board: Arc::new(board),
        })
    }
    fn row_ranges(width: usize, nthreads: usize) -> Vec<Range<usize>> {
        let slice_size = width / nthreads;
        let mut cur_left_col = 0;
        (1..=nthreads).map(|thread_idx| {
            if thread_idx == nthreads {
                cur_left_col..width
            } else {
                let range = cur_left_col..cur_left_col + slice_size;
                cur_left_col += slice_size;
                range
            }
        }).collect()
    }

    pub fn simulate(&mut self) {
        let (tx, rx) = mpsc::channel::<(Vec<Vec<Cell>>, usize)>();
        let mut thread_handles = Vec::with_capacity(self.nthreads);
        for thread_idx in 0..self.nthreads {
            let row_range = self.thread_row_ranges[thread_idx].clone();
            let board = self.board.clone();
            let tx = tx.clone();
            let thread_handle = thread::spawn(move || {
                let mut board_slice: Vec<Vec<Cell>> = Vec::with_capacity(row_range.end);
                for row_idx in row_range {
                    let mut col = Vec::with_capacity(board.height);
                    for col_idx in 0..board.height {
                        col.push(board.next_cell_state(row_idx, col_idx))
                    }
                    board_slice.push(col);
                }
                tx.send((board_slice, thread_idx)).unwrap();
            });
            thread_handles.push(thread_handle);
        }
        let mut new_gird: Vec<Vec<Cell>> = (0..self.board.width).map(|_| Vec::new()).collect();
        for handle in thread_handles {
            let _ = handle.join().expect("Threads should join correctly.");
        }
        for _ in 0..self.nthreads {
            let (board_slice, thread_idx) = rx.recv().expect("Should receive values correctly.");
            let row_range = self.thread_row_ranges[thread_idx].clone();
            for (board_col, row_idx) in board_slice.into_iter().zip(row_range) {
                new_gird[row_idx] = board_col;
            }
        }
        self.board = Arc::new(
            LifeBoard {
                grid: new_gird,
                width: self.board.width,
                height: self.board.height
            });
    }

    pub fn simulate_n_steps(&mut self, steps: u16) {
        for _ in 0..steps {
            self.simulate()
        }
    }

    pub fn is_cell_alive(&self, x: i64, y: i64) -> Option<bool> {
        self.board.is_cell_alive(x, y)
    }
} impl Debug for ParallelLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
         Debug::fmt(&self.board, f)
    }
} impl Display for ParallelLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.board, f)
    }
} impl PartialEq for ParallelLifeBoard {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.board, &other.board)
    }
}

#[cfg(test)]
mod tests {
    use crate::life::{Cell, LifeBoard, ParallelLifeBoard};
    use crate::life_interface::LifeBoardError;

    fn assert_contains(actual: String, expected: &str) {
        assert!(
            actual.contains(expected),
            "Expected \"{actual}\" to contain \"{expected}\"")
    }

    fn to_grid<A, B>(collection: A) -> Vec<Vec<Cell>>
        where
            A: IntoIterator<Item=B>,
            B: IntoIterator<Item=bool>
    {
        collection.into_iter().map(|row|
            row.into_iter().map(|alive| Cell {alive }).collect()
        ).collect()
    }

    fn get_3x3_board(array: [[bool;3];3]) -> LifeBoard {
        let board = to_grid(array);
        LifeBoard::new(board).unwrap()
    }

    #[test]
    fn test_exception_life_board_new_invalid_row() {
        let grid: Vec<Vec<Cell>> = Vec::new();
        match LifeBoard::new(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "at least one cell wide");
            }
        };
    }

    #[test]
    fn test_exception_life_board_new_invalid_col() {
        let grid: Vec<Vec<Cell>> = to_grid([[]]);
        match LifeBoard::new(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "at least one cell tall");
            }
        };
    }

    #[test]
    fn test_exception_life_board_new_inconsistent_col_len() {
        let mut grid: Vec<Vec<Cell>> = Vec::new();
        let mut col1 = Vec::<Cell>::new();
        let mut col2 = Vec::<Cell>::new();
        col1.push(Cell::gen());
        col1.push(Cell::gen());
        col2.push(Cell::gen());
        grid.push(col1);
        grid.push(col2);
        match LifeBoard::new(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "consistent size");
            }
        }
    }

    #[test]
    fn test_equivalence_life_board_new_valid_2x2_board() {
        let grid = to_grid([[false, true], [true, true]]);
        match LifeBoard::new(grid) {
            Ok(_) => (),
            Err(error) => {
                panic!("Board should be invalid. Found {error:#?}");
            }
        }
    }

    #[test]
    fn test_equivalence_life_board_is_cell_alive_false() {
        let board = LifeBoard::new(to_grid([[false]])).unwrap();
        match board.is_cell_alive(0, 0) {
            Some(alive) => assert!(!alive, "The cell should be dead."),
            None => panic!("Cell should be valid"),
        }
    }

    #[test]
    fn test_equivalence_life_board_is_cell_alive_true() {
        let board = LifeBoard::new(to_grid([[true]])).unwrap();
        match board.is_cell_alive(0, 0) {
            Some(alive) => assert!(alive, "The cell should be alive."),
            None => panic!("Cell should be valid"),
        }
    }

    #[test]
    fn test_exception_life_board_is_cell_alive_invalid_y() {
        let board = LifeBoard::new(to_grid([[true]])).unwrap();
        if let Some(_) = board.is_cell_alive(0, 1) {
            panic!("Cell should be invalid")
        }
    }

    #[test]
    fn test_exception_life_board_is_cell_alive_invalid_x() {
        let board = LifeBoard::new(to_grid([[true]])).unwrap();
        if let Some(_) = board.is_cell_alive(-1, 0) {
            panic!("Cell should be invalid")
        }
    }

    #[test]
    fn test_boundary_get_num_alive_neighbors_1x1_board() {
        let board = LifeBoard::new(to_grid([[true]])).unwrap();
        match board.get_num_alive_neighbors(0, 0) {
            0 => return,
            num => panic!("There should be no alive neighbors but found {num}."),
        }
    }

    #[test]
    fn test_equivalence_get_num_alive_neighbors_3x3_board_none() {
        let board = get_3x3_board([[false, false, false], [false, false, false], [false, false, false]]);
        match board.get_num_alive_neighbors(1, 1) {
            0 => return,
            num => panic!("There should be no alive neighbors but found {num}."),
        }
    }

    #[test]
    fn test_equivalence_get_num_alive_neighbors_3x3_board_all() {
        let board = get_3x3_board([[true, true, true], [true, false, true], [true, true, true]]);
        match board.get_num_alive_neighbors(1, 1) {
            8 => return,
            num => panic!("Expected 8 neighbors but found {num}"),
        }
    }

    #[test]
    fn test_equivalence_next_cell_state_3x3_board() {
        let board = get_3x3_board([
            [true, true, true],
            [false, true, false],
            [true, false, false]
        ]);
        assert!(board.next_cell_state(0, 0).alive, "Cell should survive");
        assert!(board.next_cell_state(0, 1).alive, "Cell should survive");
        assert!(board.next_cell_state(0, 2).alive, "Cell should survive");
        assert!(!board.next_cell_state(1, 0).alive, "Cell should remain dead");
        assert!(!board.next_cell_state(1, 1).alive, "Cell should die from overpopulation");
        assert!(board.next_cell_state(1, 2).alive, "Cell should become alive");
        assert!(!board.next_cell_state(2, 0).alive, "Cell should die from underpopulation");
        assert!(!board.next_cell_state(2, 1).alive, "Cell should remain dead");
        assert!(!board.next_cell_state(2, 2).alive, "Cell should remain dead");
    }

    fn assert_boards_eq(expected: LifeBoard, actual: LifeBoard) {
        assert_eq!(expected, actual, "\nEXPECTED:\n{expected}\n ACTUAL:\n{actual}\n")
    }

    #[test]
    fn test_equivalence_simulate_3x3_board() {
        let mut actual_board = get_3x3_start_board();
        let expected_board = get_3x3_end_board();
        actual_board = actual_board.simulate();
        assert_boards_eq(expected_board, actual_board);
    }

    fn get_3x3_end_board() -> LifeBoard {
        get_3x3_board([
            [true, true, true],
            [false, false, true],
            [false, false, false]
        ])
    }

    fn get_3x3_start_board() -> LifeBoard {
        get_3x3_board([
            [true, true, true],
            [false, true, false],
            [true, false, false]
        ])
    }

    #[test]
    fn test_equivalence_simulate_5x5_board_10_steps_all_die() {
        let mut actual_board = LifeBoard::new(to_grid([
            [true, false, false, true, false],
            [false, false, true, true, false],
            [true, true, false, false, true],
            [false, true, true, false, false],
            [true, false, false, true, false],
        ])).unwrap();
        actual_board = actual_board.simulate_n_steps(10);
        let expected_board = LifeBoard::new(to_grid([
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
        ])).unwrap();
        assert_boards_eq(expected_board, actual_board);
    }

    fn get_7x7_start_board_0th_gen() -> LifeBoard {
        LifeBoard::new(to_grid([
            [false, true, false, true, false, false, false],
            [false, true, false, false, true, false, false],
            [false, false, false, false, false, true, false],
            [false, false, false, false, false, true, false],
            [true, false, false, false, true, true, false],
            [true, false, true, false, false, false, false],
            [false, false, true, true, true, false, true],
        ])).unwrap()
    }

    fn get_7x7_board_1st_gen() -> LifeBoard {
        LifeBoard::new(to_grid([
            [false, false, true, false, false, false, false],
            [false, false, true, false, true, false, false],
            [false, false, false, false, true, true, false],
            [false, false, false, false, false, true, true],
            [false, true, false, false, true, true, false],
            [false, false, true, false, false, false, false],
            [false, true, true, true, false, false, false],
        ])).unwrap()
    }

    fn get_7x7_end_board_10th_gen() -> LifeBoard {
        LifeBoard::new(to_grid([
            [false, false, true, true, false, false, false],
            [false, false, true, true, false, false, false],
            [false, false, true, false, false, false, false],
            [false, false, true, false, false, false, false],
            [false, false, true, false, false, true, true],
            [false, false, false, false, true, false, true],
            [false, false, false, false, false, true, false],
        ])).unwrap()
    }

    #[test]
    fn test_equivalence_simulate_7x7_board_10_steps() {
        let mut actual_board = get_7x7_start_board_0th_gen();
        actual_board = actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        assert_boards_eq(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_3_threads_simulate_7x7_board_10_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from(actual_board, 3);
        actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        let expected_board = ParallelLifeBoard::from(expected_board, 3);
        assert_eq!(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_3_threads_simulate_7x7_board_1_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from(actual_board, 3);
        actual_board.simulate();
        let expected_board = get_7x7_board_1st_gen();
        let expected_board = ParallelLifeBoard::from(expected_board, 3);
        assert_eq!(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_9_threads_simulate_7x7_board_10_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from(actual_board, 9);
        actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        let expected_board = ParallelLifeBoard::from(expected_board, 9);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub alive: bool
}

impl Cell {
    pub fn gen() -> Cell {
        let alive = rand::thread_rng().gen_bool(0.5);
        Cell { alive }
    }

    pub fn new(alive: bool) -> Cell {
        Cell { alive }
    }
}
