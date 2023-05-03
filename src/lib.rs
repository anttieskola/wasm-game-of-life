mod utils;
use fixedbitset::FixedBitSet;
use js_sys;
use std::fmt;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

#[wasm_bindgen]
pub fn wasm_init() {
    utils::set_panic_hook();
}

// Memory definition (JS to access all memory)
#[wasm_bindgen] // js binding
pub fn wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}

// Universe definition
#[wasm_bindgen] // js binding
pub struct Universe {
    width: usize,
    height: usize,
    cells: FixedBitSet,
}

// wasm binded api
#[wasm_bindgen]
impl Universe {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        // Rule 1: Any live cell with fewer than two live neighbours
                        // dies, as if caused by underpopulation.
                        (true, x) if x < 2 => false,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (true, x) if x == 2 || x == 3 => true,
                        // Rule 3: Any live cell with more than three live
                        // neighbours dies, as if by overpopulation.
                        (true, x) if x > 3 => false,
                        // Rule 4: Any dead cell with exactly three live neighbours
                        // becomes a live cell, as if by reproduction.
                        (false, x) if x == 3 => true,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }
        self.cells = next;
    }
    pub fn new_random(width: usize, height: usize) -> Universe {
        let mut universe = Universe::new(height, width);
        for i in 0..(width * height) {
            let rnd = js_sys::Math::random();
            if rnd > 0.5 {
                universe.cells.set(i, true);
            }
        }
        universe
    }
    pub fn render(&self) -> String {
        self.to_string()
    }
}

// non wasm binded api
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
    fn new(width: usize, height: usize) -> Universe {
        let cells = FixedBitSet::with_capacity(width * height);
        Universe {
            width,
            height,
            cells,
        }
    }
}

// Display trait implementation
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let symbol = match self.cells[idx] {
                    true => '◼',
                    false => '◻',
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

// Testing the universe without using wasm binded API
#[cfg(test)]
mod tests {
    use super::*;

    fn set_cells(universe: &mut Universe, cells: &[(usize, usize)]) {
        for (row, col) in cells.iter() {
            let index = universe.get_index(*row, *col);
            universe.cells.set(index, true);
        }
    }

    #[test]
    fn index() {
        let universe = Universe {
            width: 64,
            height: 64,
            cells: FixedBitSet::with_capacity(64 * 64),
        };
        assert_eq!(universe.width, 64);
        assert_eq!(universe.height, 64);
        assert_eq!(universe.cells.len(), 64 * 64);
        assert_eq!(universe.get_index(0, 0), 0);
        assert_eq!(universe.get_index(1, 1), 65);
    }

    #[test]
    fn display() {
        let mut universe = Universe::new(3, 3);
        set_cells(&mut universe, &[(0, 1), (1, 1), (2, 1)]);
        let expected = String::from("◻◼◻\n◻◼◻\n◻◼◻\n");
        let result = universe.to_string();
        assert_eq!(expected, result);
    }

    #[test]
    fn tick_blinker() {
        let mut expected_universe = Universe::new(5, 5);
        set_cells(&mut expected_universe, &[(2, 1), (2, 2), (2, 3)]);
        let expected_str = expected_universe.to_string();

        let mut universe = Universe::new(5, 5);
        set_cells(&mut universe, &[(1, 2), (2, 2), (3, 2)]);
        universe.tick();
        let result_str = universe.to_string();

        assert_eq!(expected_str, result_str);
    }
}
