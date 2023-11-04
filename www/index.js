import * as wasm from "tictactoe-client";

const canvas = document.getElementById("game");
const container = document.getElementById("gameContainer");

let instance = wasm.Game.new("game");
canvas.addEventListener("mousemove", (e) => instance.on_mouse_move(e));


function render() {
    if (canvas.width !== canvas.clientWidth || canvas.height !== canvas.clientHeight) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    }
    //instance.gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

    instance.render();

    requestAnimationFrame(render);
}
requestAnimationFrame(render);