mod utils;
use fixedbitset::FixedBitSet;
use js_sys;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CELL_SIZE: f64 = 3.0;
const BORDER_SIZE: f64 = 1.0;

struct WindowInfo {
    inner_width: usize,
    inner_height: usize,
}

impl WindowInfo {
    fn new(window: &Window) -> WindowInfo {
        let width = match window.inner_width() {
            Ok(width) => width.as_f64().unwrap(),
            Err(_) => 400.0,
        };
        let height = match window.inner_height() {
            Ok(height) => height.as_f64().unwrap(),
            Err(_) => 400.0,
        };
        WindowInfo {
            inner_width: width as usize,
            inner_height: height as usize,
        }
    }
}

#[wasm_bindgen]
pub fn wasm_init() {
    utils::set_panic_hook();
}

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(closure: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(closure.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window();
    let canvas = find_canvas(&window).expect("no game-of-life-canvas found");
    let window_info = WindowInfo::new(&window);
    canvas.set_width(window_info.inner_width as u32);
    canvas.set_height(window_info.inner_height as u32);
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let mut universe = Universe::new_by_window_info(&window_info);

    ctx.begin_path();
    ctx.set_stroke_style(&JsValue::from_str("#2e2e2e"));
    let height = universe.height() as f64 * (CELL_SIZE + BORDER_SIZE) + BORDER_SIZE;
    let width = universe.width() as f64 * (CELL_SIZE + BORDER_SIZE) + BORDER_SIZE;

    // horizontal lines
    for row in 0..universe.height() {
        let spot = row as f64 * (CELL_SIZE + BORDER_SIZE);
        ctx.move_to(0.0, spot);
        ctx.line_to(width, spot);
    }

    // vertical lines
    for col in 0..universe.width() {
        let spot = col as f64 * (CELL_SIZE + BORDER_SIZE);
        ctx.move_to(spot, 0.0);
        ctx.line_to(spot, height);
    }

    ctx.stroke();

    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut tick = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if tick > 36000 {
            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }
        tick += 1;
        universe.tick();

        // draw cells
        for row in 0..universe.height() {
            for col in 0..universe.width() {
                let idx = universe.get_index(row, col);
                let cell = universe.cells[idx];
                if cell {
                    ctx.set_fill_style(&JsValue::from_str("#aeaeae"));
                } else {
                    ctx.set_fill_style(&JsValue::from_str("#000000"));
                }
                ctx.fill_rect(
                    (col as f64 * (CELL_SIZE + BORDER_SIZE)) + BORDER_SIZE,
                    (row as f64 * (CELL_SIZE + BORDER_SIZE)) + BORDER_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }
        }

        ctx.stroke();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn find_canvas(window: &Window) -> Option<HtmlCanvasElement> {
    let document = window.document()?;
    let info_element = document.get_element_by_id("game-of-life-canvas")?;
    let canvas = info_element.dyn_into::<HtmlCanvasElement>().ok()?;
    Some(canvas)
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
                        // any live cell with fewer than two live neighbours dies, as if caused by underpopulation
                        (true, x) if x < 2 => false,
                        // any live cell with two or three live neighbours lives on to the next generation
                        (true, x) if x == 2 || x == 3 => true,
                        // any live cell with more than three live neighbours dies, as if by overpopulation
                        (true, x) if x > 3 => false,
                        // any dead cell with exactly three live neighbours becomes a live cell, as if by
                        (false, x) if x == 3 => true,
                        // other cells remain in the same state
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
    fn new_by_window_info(window_info: &WindowInfo) -> Universe {
        let height = (window_info.inner_height - BORDER_SIZE as usize)
            / (CELL_SIZE as usize + BORDER_SIZE as usize);
        let width = (window_info.inner_width - BORDER_SIZE as usize)
            / (CELL_SIZE as usize + BORDER_SIZE as usize);
        let mut universe = Universe::new(width, height);
        for i in 0..(width * height) {
            let rnd = js_sys::Math::random();
            if rnd > 0.5 {
                universe.cells.set(i, true);
            }
        }
        universe
    }
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
