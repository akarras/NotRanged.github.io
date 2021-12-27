// WASM worker handles running the rust solver. Negotiating peace between the fire and water tribes.

import { threads, simd } from "https://unpkg.com/wasm-feature-detect?module";

let module;

console.log("loaded wasm worker");
var state = null;
var sim = null;
var is_thread = false;
async function start_simulator(synth) {
  console.log("init thread");
  if (await threads()) {
    if (await simd()) {
      console.log("Running SIMD solver")
      module = await import('../../lib/xiv-thread-simd/xiv_crafting_sim.js');
      await module.default();
      await module.initThreadPool(navigator.hardwareConcurrency);
      is_thread = true;
    } else {
      console.log("Running threaded solver")
      module = await import('../../lib/xiv-thread-simulator/xiv_crafting_sim.js');
      await module.default();
      await module.initThreadPool(navigator.hardwareConcurrency);
      is_thread = true;
    }
  }
  else {
      module = await import("../../lib/xiv-craft-simulator/xiv_crafting_sim.js")
      await module.default();
      console.warn("Your browser does not support threads, performance might be slower");
  }

  //sim = CraftSimulator.new_wasm(synth);
  state = {startTime: Date.now()};
  //runWasmGen();
}

self.onmessage = function(e) {
  try {
    if (e.data.start) {
      if (sim == null) {
        start_simulator(e.data.start).then(r => {
          sim = module.CraftSimulator.new_wasm(e.data.start);
          runWasmGen();
        })
      }
      else {
        sim = module.CraftSimulator.new_wasm(e.data.start);
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
      let start = Date.now();
      runWasmGen();
      console.log('took ' + (Date.now() - start) + ' ms to compute');
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
  // "finish" by doing one more generation
  let result = sim.pause_wasm();
  self.postMessage({
    success: {
      executionLog: executionLog.log,
      elapsedTime: elapsedTime,
      bestSequence: actionSequenceToShortNames(best)
    }
  });
}

