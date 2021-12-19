declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export class CraftSimulator {
	  free(): void;
	/**
	* @param {any} synth
	* @param {number} max_length
	* @param {number} population_size
	* @returns {CraftSimulator}
	*/
	  static new_wasm(synth: any, max_length: number, population_size: number): CraftSimulator;
	/**
	* @returns {any}
	*/
	  next_wasm(): any;
	}
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_craftsimulator_free: (a: number) => void;
  readonly craftsimulator_new_wasm: (a: number, b: number, c: number) => number;
  readonly craftsimulator_next_wasm: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
