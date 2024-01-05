use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::sync::{Arc, mpsc};
use std::{fmt, thread};
use rand::Rng;
use crate::life_interface::{LifeBoard, LifeBoardError, LifeCell};

#[derive(PartialEq, Clone)]
pub struct Cell { alive: bool } impl Cell {
    pub fn gen() -> Cell { Cell { alive: rand::thread_rng().gen_bool(0.5) } }

    pub fn new(alive: bool) -> Cell { Cell { alive } }
} impl LifeCell for Cell {
    fn is_alive(&self) -> bool { self.alive }
}

#[derive(PartialEq, Clone)]
pub struct BaseLifeBoard { grid: Vec<Vec<Cell>>, width: usize, height: usize, } impl BaseLifeBoard {
    fn from_bool_matrix<A, B>(collection: A) -> Result<BaseLifeBoard, LifeBoardError>
        where
            A: IntoIterator<Item=B>,
            B: IntoIterator<Item=bool>
    {
        let grid = collection.into_iter().map(|row|
            row.into_iter().map(|alive|
                Cell { alive }
            ).collect()
        ).collect();
        return BaseLifeBoard::_from_grid(grid);
    }

    fn from_cell_matrix<A, B>(collection: A) -> Result<BaseLifeBoard, LifeBoardError>
        where
            A: IntoIterator<Item=B>,
            B: IntoIterator<Item=Cell>
    {
        let grid = collection.into_iter().map(|row|
            row.into_iter().collect()
        ).collect();
        return BaseLifeBoard::_from_grid(grid);
    }

    fn _from_grid(grid: Vec<Vec<Cell>>) -> Result<BaseLifeBoard, LifeBoardError> {
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
        return Ok(BaseLifeBoard { grid, width, height })
    }

    pub fn gen(width: usize, height: usize) -> BaseLifeBoard {
        let mut grid: Vec<Vec<Cell>> = Vec::with_capacity(width);
        for _ in 0..width {
            let mut col = Vec::with_capacity(height);
            for _ in 0..height {
                col.push(Cell::gen());
            }
            grid.push(col);
        }

        BaseLifeBoard { grid, width, height }
    }

    fn _is_cell_alive(&self, x: i64, y: i64) -> Option<bool> {
        self._cell_at(x, y).map(|cell| cell.alive)
    }

    fn _cell_at(&self, x: i64, y: i64) -> Option<Cell> {
        let (x, y) = match (x, y) {
            (x, _) if x < 0 => return None,
            (_, y) if y < 0 => return None,
            _ => (x as usize, y as usize),
        };
        match self.grid.get(x) {
            Some(row) => match row.get(y) {
                Some(cell) => Some(cell.clone()),
                None => None
            },
            None => None
        }
    }

    fn into_vec_matrix(self) -> Vec<Vec<Cell>> { self.grid }

    fn _board_fmt(&self, f: &mut Formatter<'_>, alive_cell: &str, dead_cell: &str, dbg: bool) -> fmt::Result {
        for col_idx in 0..self.height() {
            for row_idx in 0..self.width() {
                let cell = self.cell_at(row_idx, col_idx).expect("Should always be valid indices");
                let alive = if cell.alive { alive_cell } else { dead_cell };
                let cell_string = if dbg {
                    format!("({alive}, {row_idx}, {col_idx}) ")
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
} impl LifeBoard<Cell> for BaseLifeBoard {
    fn width(&self) -> usize { self.width }
    fn height(&self) -> usize { self.height }

    fn simulate(&mut self) {
        let mut new_grid: Vec<Vec<Cell>> = Vec::with_capacity(self.width);
        for row_idx in 0..self.width {
            let mut new_col = Vec::with_capacity(self.height);
            for col_idx in 0..self.height {
                let new_cell = self.next_cell_state_at(row_idx, col_idx)
                    .expect("Should always access a valid index");
                new_col.push(new_cell);
            }
            new_grid.push(new_col);
        }
        self.grid = new_grid;
    }

    fn simulate_n_steps(&mut self, n: usize) {
        for _ in 0..n {
            self.simulate();
        }
    }

    fn next_cell_state_at(&self, x: usize, y: usize) -> Option<Cell> {
        let old_cell = match self.cell_at(x, y) {
            Some(cell) => cell,
            None => return None,
        };
        let alive = match self.num_alive_neighbors_at(x, y) {
            0|1 if old_cell.alive => false,
            2|3 if old_cell.alive => true,
            4..=8 if old_cell.alive => false,
            3 if !old_cell.alive => true,
            _ => false,
        };
        Some(Cell { alive })
    }

    fn cell_at(&self, x: usize, y: usize) -> Option<Cell> { self._cell_at(x as i64, y as i64) }

    fn num_alive_neighbors_at(&self, x: usize, y: usize) -> u8 {
        let mut neighbors = 0u8;
        for dx in 0..3 {
            for dy in 0..3 {
                if dx == 1 && dy == 1 {
                    continue
                } else {
                    let (x_test, y_test) = ((x as i64 - 1) + dx, (y as i64 - 1) + dy);
                    if let Some(is_alive) = self._is_cell_alive(x_test, y_test) {
                        if is_alive { neighbors += 1; }
                    }
                }
            }
        }
        neighbors
    }

    fn is_cell_alive(&self, x: usize, y: usize) -> Option<bool> { self._is_cell_alive(x as i64, y as i64) }

    fn to_vec_matrix(&self) -> Vec<Vec<Cell>> { self.grid.clone() }
} impl Display for BaseLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self._board_fmt(f, "*", " ", false)
    }
} impl Debug for BaseLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self._board_fmt(f, "T", "F", true)
    }
}

#[derive(PartialEq, Clone)]
pub struct ParallelLifeBoard {
    board: Arc<BaseLifeBoard>,
    n_threads: usize,
    thread_row_ranges: Vec<Range<usize>>,
} impl ParallelLifeBoard {
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

    pub fn from_matrix<A, B>(collection: A, n_threads: u8) -> Result<ParallelLifeBoard, LifeBoardError>
        where
            A: IntoIterator<Item=B>,
            B: IntoIterator<Item=bool>
    {
        let board = BaseLifeBoard::from_bool_matrix(collection);
        board.map(|board|
            ParallelLifeBoard {
                thread_row_ranges: ParallelLifeBoard::row_ranges(board.width, n_threads as usize),
                n_threads: n_threads as usize,
                board: Arc::new(board),
            }
        )
    }

    pub fn from_board(board: BaseLifeBoard, n_threads: u8) -> ParallelLifeBoard {
        ParallelLifeBoard {
            thread_row_ranges: ParallelLifeBoard::row_ranges(board.width, n_threads as usize),
            n_threads: n_threads as usize,
            board: Arc::new(board),
        }
    }

    fn _from_grid(grid: Vec<Vec<Cell>>, n_threads: u8) -> Result<ParallelLifeBoard, LifeBoardError> {
        let board = BaseLifeBoard::_from_grid(grid);
        board.map(|board|
            ParallelLifeBoard {
                thread_row_ranges: ParallelLifeBoard::row_ranges(board.width, n_threads as usize),
                n_threads: n_threads as usize,
                board: Arc::new(board),
            }
        )
    }

    pub fn gen(width: usize, height: usize, n_threads: u8) -> ParallelLifeBoard {
        let board = BaseLifeBoard::gen(width, height);
        ParallelLifeBoard {
            thread_row_ranges: ParallelLifeBoard::row_ranges(width, n_threads as usize),
            n_threads: n_threads as usize,
            board: Arc::new(board),
        }
    }

    fn _is_cell_alive(&self, x: i64, y: i64) -> Option<bool> {
        self.board._cell_at(x, y).map(|cell| cell.alive)
    }

    fn _cell_at(&self, x: i64, y: i64) -> Option<Cell> {
        self.board._cell_at(x, y)
    }
} impl LifeBoard<Cell> for ParallelLifeBoard {
    fn width(&self) -> usize { self.board.width }

    fn height(&self) -> usize { self.board.height }

    fn simulate(&mut self) {
        let (tx, rx) = mpsc::channel::<(Vec<Vec<Cell>>, usize)>();
        let mut thread_handles = Vec::with_capacity(self.n_threads);
        for thread_idx in 0..self.n_threads {
            let row_range = self.thread_row_ranges[thread_idx].clone();
            let board = self.board.clone();
            let tx = tx.clone();
            let thread_handle = thread::spawn(move || {
                let mut board_slice: Vec<Vec<Cell>> = Vec::with_capacity(row_range.end);
                for row_idx in row_range {
                    let mut col = Vec::with_capacity(board.height);
                    for col_idx in 0..board.height {
                        col.push(
                            board.next_cell_state_at(row_idx, col_idx)
                                .expect("Should always be valid indexes")
                        )
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
        for _ in 0..self.n_threads {
            let (board_slice, thread_idx) = rx.recv().expect("Should receive values correctly.");
            let row_range = self.thread_row_ranges[thread_idx].clone();
            for (board_col, row_idx) in board_slice.into_iter().zip(row_range) {
                new_gird[row_idx] = board_col;
            }
        }
        self.board = Arc::new(
            BaseLifeBoard {
                grid: new_gird,
                width: self.board.width,
                height: self.board.height
            });
    }

    fn simulate_n_steps(&mut self, steps: usize) {
        for _ in 0..steps {
            self.simulate()
        }
    }

    fn next_cell_state_at(&self, x: usize, y: usize) -> Option<Cell> { self.board.next_cell_state_at(x, y) }

    fn cell_at(&self, x: usize, y: usize) -> Option<Cell> { self.board.cell_at(x, y) }

    fn num_alive_neighbors_at(&self, x: usize, y: usize) -> u8 { self.board.num_alive_neighbors_at(x, y) }

    fn is_cell_alive(&self, x: usize, y: usize) -> Option<bool> { self.board.is_cell_alive(x, y) }

    fn to_vec_matrix(&self) -> Vec<Vec<Cell>> { self.board.grid.clone() }
} impl Debug for ParallelLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.board, f)
    }
} impl Display for ParallelLifeBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.board, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::life_implementation::{BaseLifeBoard, Cell, ParallelLifeBoard};
    use crate::life_interface::{LifeBoard, LifeBoardError};

    fn assert_contains(actual: String, expected: &str) {
        assert!(
            actual.contains(expected),
            "Expected \"{actual}\" to contain \"{expected}\"")
    }

    fn get_3x3_board(array: [[bool;3];3]) -> BaseLifeBoard {
        BaseLifeBoard::from_bool_matrix(array).unwrap()
    }

    #[test]
    fn test_exception_life_board_from_matrix_invalid_row() {
        let grid: Vec<Vec<Cell>> = Vec::new();
        match BaseLifeBoard::from_cell_matrix(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "at least one cell wide");
            },
            Err(error) => panic!("Unexpected LifeBoardError {error:?}"),
        };
    }

    #[test]
    fn test_exception_life_board_from_matrix_invalid_col() {
        match BaseLifeBoard::from_bool_matrix([[]]) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "at least one cell tall");
            },
            Err(error) => panic!("Unexpected LifeBoardError {error:?}"),
        };
    }

    #[test]
    fn test_exception_life_board_from_matrix_inconsistent_col_len() {
        let mut grid: Vec<Vec<Cell>> = Vec::new();
        let mut col1 = Vec::<Cell>::new();
        let mut col2 = Vec::<Cell>::new();
        col1.push(Cell::gen());
        col1.push(Cell::gen());
        col2.push(Cell::gen());
        grid.push(col1);
        grid.push(col2);
        match BaseLifeBoard::from_cell_matrix(grid) {
            Ok(_) => panic!("Board should be invalid."),
            Err(LifeBoardError::InvalidBoard(error)) => {
                assert_contains(error, "consistent size");
            },
            Err(error) => panic!("Unexpected LifeBoardError {error:?}"),
        }
    }

    #[test]
    fn test_equivalence_life_board_from_matrix_valid_2x2_board() {
        match BaseLifeBoard::from_bool_matrix([[false, true], [true, true]]) {
            Ok(_) => (),
            Err(error) => {
                panic!("Board should be invalid. Found {error:#?}");
            }
        }
    }

    #[test]
    fn test_equivalence_life_board_is_cell_alive_false() {
        let board = BaseLifeBoard::from_bool_matrix([[false]]).unwrap();
        match board.is_cell_alive(0, 0) {
            Some(alive) => assert!(!alive, "The cell should be dead."),
            None => panic!("Cell should be valid"),
        }
    }

    #[test]
    fn test_equivalence_life_board_is_cell_alive_true() {
        let board = BaseLifeBoard::from_bool_matrix([[true]]).unwrap();
        match board.is_cell_alive(0, 0) {
            Some(alive) => assert!(alive, "The cell should be alive."),
            None => panic!("Cell should be valid"),
        }
    }

    #[test]
    fn test_exception_life_board_is_cell_alive_invalid_y() {
        let board = BaseLifeBoard::from_bool_matrix([[true]]).unwrap();
        if let Some(_) = board.is_cell_alive(0, 1) {
            panic!("Cell should be invalid")
        }
    }

    #[test]
    fn test_exception_life_board_is_cell_alive_invalid_x() {
        let board = BaseLifeBoard::from_bool_matrix([[true]]).unwrap();
        if let Some(_) = board._is_cell_alive(-1, 0) {
            panic!("Cell should be invalid")
        }
    }

    #[test]
    fn test_boundary_get_num_alive_neighbors_1x1_board() {
        let board = BaseLifeBoard::from_bool_matrix([[true]]).unwrap();
        match board.num_alive_neighbors_at(0, 0) {
            0 => return,
            num => panic!("There should be no alive neighbors but found {num}."),
        }
    }

    #[test]
    fn test_equivalence_get_num_alive_neighbors_3x3_board_none() {
        let board = get_3x3_board([[false, false, false], [false, false, false], [false, false, false]]);
        match board.num_alive_neighbors_at(1, 1) {
            0 => return,
            num => panic!("There should be no alive neighbors but found {num}."),
        }
    }

    #[test]
    fn test_equivalence_get_num_alive_neighbors_3x3_board_all() {
        let board = get_3x3_board([[true, true, true], [true, false, true], [true, true, true]]);
        match board.num_alive_neighbors_at(1, 1) {
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
        assert!(board.next_cell_state_at(0, 0).unwrap().alive, "Cell should survive");
        assert!(board.next_cell_state_at(0, 1).unwrap().alive, "Cell should survive");
        assert!(board.next_cell_state_at(0, 2).unwrap().alive, "Cell should survive");
        assert!(!board.next_cell_state_at(1, 0).unwrap().alive, "Cell should remain dead");
        assert!(!board.next_cell_state_at(1, 1).unwrap().alive, "Cell should die from overpopulation");
        assert!(board.next_cell_state_at(1, 2).unwrap().alive, "Cell should become alive");
        assert!(!board.next_cell_state_at(2, 0).unwrap().alive, "Cell should die from underpopulation");
        assert!(!board.next_cell_state_at(2, 1).unwrap().alive, "Cell should remain dead");
        assert!(!board.next_cell_state_at(2, 2).unwrap().alive, "Cell should remain dead");
    }

    fn assert_boards_eq(expected: BaseLifeBoard, actual: BaseLifeBoard) {
        assert_eq!(expected, actual, "\nEXPECTED:\n{expected}\n ACTUAL:\n{actual}\n")
    }

    #[test]
    fn test_equivalence_simulate_3x3_board() {
        let mut actual_board = get_3x3_start_board();
        let expected_board = get_3x3_end_board();
        actual_board.simulate();
        assert_boards_eq(expected_board, actual_board);
    }

    fn get_3x3_end_board() -> BaseLifeBoard {
        get_3x3_board([
            [true, true, true],
            [false, false, true],
            [false, false, false]
        ])
    }

    fn get_3x3_start_board() -> BaseLifeBoard {
        get_3x3_board([
            [true, true, true],
            [false, true, false],
            [true, false, false]
        ])
    }

    #[test]
    fn test_equivalence_simulate_5x5_board_10_steps_all_die() {
        let mut actual_board = BaseLifeBoard::from_bool_matrix([
            [true, false, false, true, false],
            [false, false, true, true, false],
            [true, true, false, false, true],
            [false, true, true, false, false],
            [true, false, false, true, false],
        ]).unwrap();
        actual_board.simulate_n_steps(10);
        let expected_board = BaseLifeBoard::from_bool_matrix([
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
        ]).unwrap();
        assert_boards_eq(expected_board, actual_board);
    }

    fn get_7x7_start_board_0th_gen() -> BaseLifeBoard {
        BaseLifeBoard::from_bool_matrix([
            [false, true, false, true, false, false, false],
            [false, true, false, false, true, false, false],
            [false, false, false, false, false, true, false],
            [false, false, false, false, false, true, false],
            [true, false, false, false, true, true, false],
            [true, false, true, false, false, false, false],
            [false, false, true, true, true, false, true],
        ]).unwrap()
    }

    fn get_7x7_board_1st_gen() -> BaseLifeBoard {
        BaseLifeBoard::from_bool_matrix([
            [false, false, true, false, false, false, false],
            [false, false, true, false, true, false, false],
            [false, false, false, false, true, true, false],
            [false, false, false, false, false, true, true],
            [false, true, false, false, true, true, false],
            [false, false, true, false, false, false, false],
            [false, true, true, true, false, false, false],
        ]).unwrap()
    }

    fn get_7x7_end_board_10th_gen() -> BaseLifeBoard {
        BaseLifeBoard::from_bool_matrix([
            [false, false, true, true, false, false, false],
            [false, false, true, true, false, false, false],
            [false, false, true, false, false, false, false],
            [false, false, true, false, false, false, false],
            [false, false, true, false, false, true, true],
            [false, false, false, false, true, false, true],
            [false, false, false, false, false, true, false],
        ]).unwrap()
    }

    #[test]
    fn test_equivalence_simulate_7x7_board_10_steps() {
        let mut actual_board = get_7x7_start_board_0th_gen();
        actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        assert_boards_eq(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_3_threads_simulate_7x7_board_10_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from_board(actual_board, 3);
        actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        let expected_board = ParallelLifeBoard::from_board(expected_board, 3);
        assert_eq!(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_3_threads_simulate_7x7_board_1_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from_board(actual_board, 3);
        actual_board.simulate();
        let expected_board = get_7x7_board_1st_gen();
        let expected_board = ParallelLifeBoard::from_board(expected_board, 3);
        assert_eq!(expected_board, actual_board);
    }

    #[test]
    fn test_equivalence_parallel_9_threads_simulate_7x7_board_10_steps() {
        let actual_board = get_7x7_start_board_0th_gen();
        let mut actual_board = ParallelLifeBoard::from_board(actual_board, 9);
        actual_board.simulate_n_steps(10);
        let expected_board = get_7x7_end_board_10th_gen();
        let expected_board = ParallelLifeBoard::from_board(expected_board, 9);
    }
}