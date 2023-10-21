import * as wasm from "tictactoe-client";

const canvas = document.getElementById("game");
const container = document.getElementById("gameContainer");

const gl = wasm.init_webgl("game");
const shader_program = wasm.init_shaders(gl);

let prevTime = performance.now();
let frames = 0;

function render() {
    if (canvas.width !== canvas.clientWidth || canvas.height !== canvas.clientHeight) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    }
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    //wasm.draw_triangle(gl, shader_program);
    wasm.draw_grid(gl, shader_program, 10, 10);
    wasm.render();

    if (frames % 100 == 0 && frames != 0) {
        const currentTime = performance.now();
        const elapsedTime = currentTime - prevTime;
        const fps = 1000 / elapsedTime * 100;

        console.log("FPS: ", fps);

        prevTime = currentTime;
    }

    frames++;

    requestAnimationFrame(render);
}
requestAnimationFrame(render);