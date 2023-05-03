# Repository for doing tutorial with webassembly & rust
- [Tutorial](https://rustwasm.github.io/docs/book/game-of-life/introduction.html)
- [Wikipedia](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life)

# Restrictions
- When API is decorated with wasm_bindgen it can't be called from non-wasm 
target, for example can't write a regular unit test using them
    - Exception is that if funtion has no return type and no parameters it seems to work for example `tick()`

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
