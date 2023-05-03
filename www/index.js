import { Universe, wasm_init, wasm_memory } from "wasm-game-of-life";
wasm_init();

const CELL_SIZE = 3; // px
const GRID_COLOR = "#595959";
const DEAD_COLOR = "#c96a29";
const ALIVE_COLOR = "#04fa58";

const height = 32;
const width = 32;
const universe = Universe.new_random(height, width);

const canvas = document.getElementById("game-of-life-canvas");
const text = document.getElementById("game-of-life-text");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');
const renderLoop = () => {
    universe.tick();
    drawGrid();
    drawCells();
    text.textContent = universe.render();
    requestAnimationFrame(renderLoop);
};

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
};

const getIndex = (row, column) => {
    return row * width + column;
};

const drawCells = () => {
    const cellsPtr = universe.cells();
    const cells = new Uint8Array(wasm_memory().buffer, cellsPtr, width * height / 8);

    ctx.beginPath();

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            // index of the cell in the universe
            const idx = getIndex(row, col);
            // our bit index array is represented as an array of 8 bit integers
            // each integer represents 8 cells, so we need to find the correct
            // integer from the array first
            const integerIndex = Math.floor(idx / 8);
            // bitmask to find the correct bit in the integer from the array
            const integerBitMask = 1 << (idx % 8);
            // check if the bit is set in the integer
            const isBitSet = (cells[integerIndex] & integerBitMask) === integerBitMask;
            // set bit alive/dead
            ctx.fillStyle = isBitSet
                ? ALIVE_COLOR
                : DEAD_COLOR;
            // fill cell
            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE
            );
        }
    }
    ctx.stroke();
};

drawGrid();
drawCells();
requestAnimationFrame(renderLoop);
