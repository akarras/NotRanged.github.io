// NEW TEST

import init, {initThreadPool, CraftSimulator} from "../../lib/pkg/xiv_crafting_sim.js";

//importScripts("../../lib/pkg/xiv_crafting_sim.js");
//importScripts('../../lib/string/String.js');
console.log("loaded wasm worker");
var state = null;
var sim = null;
async function start_simulator(synth) {
  console.log("init thread");
  await init();
  await initThreadPool(navigator.hardwareConcurrency);

  //sim = CraftSimulator.new_wasm(synth);
  state = {startTime: Date.now()};
  //runWasmGen();
}

self.onmessage = function(e) {
  try {
    if (e.data.start) {
      if (sim == null) {
        start_simulator(e.data.start).then(r => {
          sim = CraftSimulator.new_wasm(e.data.start);
          runWasmGen();
        })
      }
      else {
        sim = CraftSimulator.new_wasm(e.data.start);
        runWasmGen();
      }

    }
    else if (e.data == 'resume') {
      if (state.gen >= state.maxGen) {
        state.maxGen += state.settings.solver.generations;
      }
      runWasmGen();
    }
    else if (e.data == 'rungen') {
      console.log("Started ", Date.now());
      runWasmGen();
      console.log("Finish ", Date.now());
      // runOneGen();
    }
    else if (e.data == 'finish') {
      finish();
    }
  } catch (ex) {
    console.error(ex);
    self.postMessage({
      error: {
        error: ex.toString(),
        executionLog: state.logOutput && state.logOutput.log
      }
    })
  }
};

function runWasmGen() {
  let result = sim.next_wasm();
  console.log(result);
  postWasmMessage(result);
}

function postWasmMessage(result) {
  self.postMessage(result)
}

function finish() {
  var elapsedTime = Date.now() - state.startTime;

  var executionLog = state.logOutput;
  var best = state.hof.entries[0];

  self.postMessage({
    success: {
      executionLog: executionLog.log,
      elapsedTime: elapsedTime,
      bestSequence: actionSequenceToShortNames(best)
    }
  });
}

