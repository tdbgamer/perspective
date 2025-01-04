// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

// @ts-ignore
import * as perspective_server from "../../../dist/pkg/perspective-server.js";

export type * from "../../../dist/pkg/perspective-js.js";

export type PspPtr = bigint | number;

export interface EmscriptenServer {}
export interface EmscriptenApi {
    HEAP8: Int8Array;
    HEAPU8: Uint8Array;
    HEAP16: Int16Array;
    HEAPU16: Uint16Array;
    HEAP32: Int32Array;
    HEAPU32: Uint32Array;
    HEAPU64: BigUint64Array;
    _psp_alloc(size: PspPtr): PspPtr;
    _psp_free(ptr: PspPtr): void;
    _psp_new_server(): EmscriptenServer;
    _psp_delete_server(server: EmscriptenServer): void;
    _psp_is_memory64(): boolean;
    _psp_handle_request(
        server: EmscriptenServer,
        client_id: number,
        buffer_ptr: PspPtr,
        buffer_len: PspPtr
    ): PspPtr;
    _psp_poll(server: EmscriptenServer): PspPtr;
    _psp_new_session(server: EmscriptenServer): number;
    _psp_close_session(server: EmscriptenServer, client_id: number): void;
}

export async function compile_perspective(
    wasmBinary: ArrayBuffer
): Promise<EmscriptenApi> {
    const module: EmscriptenApi = await perspective_server.default({
        locateFile(x: any) {
            return x;
        },
        instantiateWasm: async (
            imports: any,
            receive: (_: WebAssembly.Instance) => void
        ) => {
            imports["env"] = {
                ...imports["env"],
                psp_stack_trace() {
                    const str = Error().stack || "";
                    const textEncoder = new TextEncoder();
                    const bytes = textEncoder.encode(str);
                    const ptr = module._psp_alloc(
                        module._psp_is_memory64()
                            ? BigInt(bytes.byteLength + 1)
                            : bytes.byteLength + 1
                    );

                    module.HEAPU8.set(bytes, Number(ptr));
                    module.HEAPU8[Number(ptr) + bytes.byteLength] = 0;
                    return ptr;
                },
                psp_heap_size() {
                    return module.HEAP8.buffer.byteLength;
                },
            };

            const webasm = await WebAssembly.instantiate(wasmBinary, imports);
            receive(webasm.instance);
            return webasm.instance.exports;
        },
    });

    return module;
}
