/* tslint:disable */
/* eslint-disable */

export class BookData {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    author: string;
    errors: string;
    title: string;
    xhtmls: string[];
}

export class StandaloneXHTML {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    occured_error: string;
    result: string;
}

export function build_epub_bytes(from: Uint8Array, styles: string[], encoding: string): Uint8Array;

export function generate_standalone_xhtml(from: string, delimiter: string): StandaloneXHTML;

export function get_ayame_css(name: string): string | undefined;

export function init_panic_hook(): void;

export function parse_to_book_data(from: string): BookData;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly get_ayame_css: (a: number, b: number) => [number, number];
    readonly __wbg_standalonexhtml_free: (a: number, b: number) => void;
    readonly __wbg_get_standalonexhtml_result: (a: number) => [number, number];
    readonly __wbg_get_standalonexhtml_occured_error: (a: number) => [number, number];
    readonly generate_standalone_xhtml: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly __wbg_bookdata_free: (a: number, b: number) => void;
    readonly __wbg_get_bookdata_title: (a: number) => [number, number];
    readonly __wbg_set_bookdata_title: (a: number, b: number, c: number) => void;
    readonly __wbg_get_bookdata_author: (a: number) => [number, number];
    readonly __wbg_set_bookdata_author: (a: number, b: number, c: number) => void;
    readonly __wbg_get_bookdata_xhtmls: (a: number) => [number, number];
    readonly __wbg_set_bookdata_xhtmls: (a: number, b: number, c: number) => void;
    readonly __wbg_get_bookdata_errors: (a: number) => [number, number];
    readonly __wbg_set_bookdata_errors: (a: number, b: number, c: number) => void;
    readonly parse_to_book_data: (a: number, b: number) => [number, number, number];
    readonly build_epub_bytes: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly init_panic_hook: () => void;
    readonly __wbg_set_standalonexhtml_result: (a: number, b: number, c: number) => void;
    readonly __wbg_set_standalonexhtml_occured_error: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_alloc: () => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
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
