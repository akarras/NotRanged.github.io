// WASM worker handles running the rust solver. Negotiating peace between the fire and water tribes.

// firefox's polyfill worker is going to look for this script in it's local directory, so I just put it in two places so that I can stop figuring this out.
import { threads } from './wasm-feature-detect.js';
import { simd } from './wasm-feature-detect.js';

let module;

console.log("loaded wasm worker");
var state = null;
var sim = null;
var is_thread = false;
let last_run = Date.now();
async function load_simd_solver() {
  module = await import('../../lib/xiv-thread-simd/xiv_crafting_sim.js');
  console.log("Running SIMD solver");
  await module.default();
  await module.initThreadPool(navigator.hardwareConcurrency);
  is_thread = true;
}

async function load_thread_solver() {
  console.log("Running threaded solver");
  module = await import('../../lib/xiv-thread-simulator/xiv_crafting_sim.js');
  await module.default();
  await module.initThreadPool(navigator.hardwareConcurrency);
  is_thread = true;
}

async function start_simulator() {
  if (threads === undefined) {
    console.warn("Unable to detect platform details. Going to yolo it and try to load the simd solver since most browsers support that anyways");
    await load_thread_solver();
    return;
  }
  if (threads !== undefined && await threads()) {
    if (await simd()) {
      await load_simd_solver();
    } else {
      await load_thread_solver();
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
      console.log('time since last run ' + (Date.now() - last_run));
      let start = Date.now();
      runWasmGen();
      console.log('took ' + (Date.now() - start) + ' ms to compute');
      last_run = Date.now();
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
  self.postMessage(result)
}

function finish() {
  // "finish" by doing one more generation
  let result = sim.pause_wasm();
  console.log(result);
  self.postMessage(result)
}

