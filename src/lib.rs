mod utils;
use wasm_bindgen::prelude::*;
use std::fmt;
use getrandom;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-game-of-life!");
}


/*
https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life

The Game of Life, also known simply as Life, is a cellular automaton devised by the British mathematician John Horton Conway in 1970.
The "game" is a zero-player game, meaning that its evolution is determined by its initial state, requiring no further input.
One interacts with the Game of Life by creating an initial configuration and observing how it evolves.
It is Turing complete and can simulate a universal constructor or any other Turing machine.

1. any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
2. any live cell with two or three live neighbours lives on to the next generation.
3. any live cell with more than three live neighbours dies, as if by overpopulation.
4. any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
 */

#[wasm_bindgen] // js binding
#[repr(u8)] // each cell is represented by a single byte
#[derive(Clone, Copy, Debug, PartialEq, Eq)] // common traits
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen] // js binding
pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Universe {
    fn get_index(&self, row: usize, column: usize) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: usize, column: usize) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn get_height(&self) -> usize {
        self.height
    }
    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };
                next[idx] = next_cell;
            }
        }
        self.cells = next;
    }

    pub fn new() -> Universe {
        let width: usize = 64;
        let height: usize = 64;

        // random seed
        let cells: Vec<Cell> = (0..width * height)
            .map(|_i| {
                let mut rb = [0u8; 1];
                match getrandom::getrandom(&mut rb) {
                    Ok(()) => {
                        if rb[0] > 128 {
                            Cell::Alive
                        } else {
                            Cell::Dead
                        }
                    },
                    Err(e) => panic!("Error: {}", e),
                }
            }
            ).collect();
        
        // hacker emblem
        // let mut cells: Vec<Cell> = Vec::with_capacity(width * height);
        // [1, 3]
        // [2, 1]
        // [2, 3]
        // [3, 2]
        // [3, 3]
        // cells[1 * width + 3] = Cell::Alive;
        // cells[2 * width + 1] = Cell::Alive;
        // cells[2 * width + 3] = Cell::Alive;
        // cells[3 * width + 2] = Cell::Alive;
        // cells[3 * width + 3] = Cell::Alive;

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }    
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
