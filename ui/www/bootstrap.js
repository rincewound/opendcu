import {
  tada,
	default as init
} from '../wasm/ui.js';

// this function is needed to initiate the wasm lib
async function run() {
  await init('../wasm/ui_bg.wasm');
}
run();

// the lib could be used after calling the run function
document.getElementById("but").onclick = function() { tada() }
