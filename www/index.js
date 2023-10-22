import * as wasm from "tictactoe-client";

const canvas = document.getElementById("game");
const container = document.getElementById("gameContainer");

let prevTime = performance.now();

let instance = wasm.Game.new("game");
canvas.addEventListener("mousemove", (e) => instance.on_mouse_move(e));

let frames = 0;

function render() {
    if (canvas.width !== canvas.clientWidth || canvas.height !== canvas.clientHeight) {
        canvas.width = canvas.clientWidth;
        canvas.height = canvas.clientHeight;
    }
    //instance.gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

    instance.render();

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