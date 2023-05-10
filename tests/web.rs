//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

extern crate wasm_game_of_life;
use wasm_game_of_life::Universe;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn new_random() {
    let universe = Universe::new_random(12, 12);
    assert_eq!(universe.width(), 12);
    assert_eq!(universe.height(), 12);
}

#[wasm_bindgen_test]
fn tick() {
    let mut universe = Universe::new_random(12, 12);
    let original = universe.to_string();
    universe.tick();
    let next = universe.to_string();
    assert_ne!(original, next);
}
