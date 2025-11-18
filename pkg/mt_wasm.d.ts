/* tslint:disable */
/* eslint-disable */
export class Buffer {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  random_gen(seed: number): void;
  set_first_state(state: Uint32Array, n: number): void;
  set_op(op: Uint32Array): void;
  to_string(): string;
  update_applications(applications: Uint32Array): void;
  now_state_ptr(): number;
  first_state_ptr(): number;
  b_state_ptr(): number;
  set_time(time: number): boolean;
  get_time(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_buffer_free: (a: number, b: number) => void;
  readonly buffer_new: () => number;
  readonly buffer_random_gen: (a: number, b: number) => void;
  readonly buffer_set_first_state: (a: number, b: number, c: number, d: number) => void;
  readonly buffer_set_op: (a: number, b: number, c: number) => void;
  readonly buffer_to_string: (a: number) => [number, number];
  readonly buffer_update_applications: (a: number, b: number, c: number) => void;
  readonly buffer_now_state_ptr: (a: number) => number;
  readonly buffer_first_state_ptr: (a: number) => number;
  readonly buffer_b_state_ptr: (a: number) => number;
  readonly buffer_set_time: (a: number, b: number) => number;
  readonly buffer_get_time: (a: number) => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
