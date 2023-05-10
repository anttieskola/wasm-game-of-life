# Repository for doing game of life tutorial with rust + webassembly
- [Tutorial](https://rustwasm.github.io/docs/book/game-of-life/introduction.html)
- [Wikipedia](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life)

## Summary
The Game of Life, also known simply as Life, is a cellular automaton devised by the British mathematician John Horton Conway in 1970.
The "game" is a zero-player game, meaning that its evolution is determined by its initial state, requiring no further input.
One interacts with the Game of Life by creating an initial configuration and observing how it evolves.
It is Turing complete and can simulate a universal constructor or any other Turing machine.

1. any live cell with fewer than two live neighbours dies, as if caused by underpopulation.
2. any live cell with two or three live neighbours lives on to the next generation.
3. any live cell with more than three live neighbours dies, as if by overpopulation.
4. any dead cell with exactly three live neighbours becomes a live cell, as if by  

# Restrictions
- When API is decorated with wasm_bindgen it can't be called from non-wasm 
target, for example can't write a regular unit test using them
    - Exception is that if funtion has no return type and no parameters it seems to work for example `tick()`

# Building
```bash
# in root

# only rust parts
cargo b
# rust only unit tests
cargo t

# web assembly build
wasm-pack build
# web assembly binding tests
wasm-pack test --firefox --headless

# in www folder
# run dev server
npm run start
# build release
npm run build
# this builds deployable files into www/dist folder
```

# Wasm-bindgen
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/)

# Crates
- [getrandom](https://crates.io/crates/getrandom)
- [js-sys](https://crates.io/crates/js-sys)
- [fixedbitset](https://crates.io/crates/fixedbitset)
- [web-sys](https://crates.io/crates/web-sys)

# Templates used
- [Rust app](./Template-README.md)
- [Web app](./www/README.md)

# Optimization

## Used optimizations
- wee_alloc memory allocator (slow but small memory allocator)
- profile release optimization lto (link-time optimization) enabled

Total application size ~14KB

## Binaryen
[Homepage](https://github.com/WebAssembly/binaryen)

Downloaded and compiled the tool, then did try this to optimize
```bash
wasm-opt -Os -o wasm_game_of_life_bg.wasm.optimized wasm_game_of_life_bg.wasm
wasm-opt -O -o wasm_game_of_life_bg.wasm.optimized wasm_game_of_life_bg.wasm
```
But results were like under 1KB reduction of size so not using it for this.

## Gzip
Enabling gzip in nginx reduces size in half (nginx) (28KB -> 14KB)

