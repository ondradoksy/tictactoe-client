import * as wasm from "tictactoe-client";

const canvas = document.getElementById("game");
const container = document.getElementById("gameContainer");

const gl = wasm.init_webgl("game");
const shader_program = wasm.init_shaders(gl);

function render() {
    if (canvas.width !== canvas.clientWidth || canvas.height !== canvas.clientHeight) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    }
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    wasm.draw_triangle(gl, shader_program);
    wasm.render();
    requestAnimationFrame(render);
}
requestAnimationFrame(render);