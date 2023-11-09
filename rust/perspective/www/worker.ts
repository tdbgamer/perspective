import {
    WASI,
    File,
    OpenFile,
    PreopenDirectory,
} from "@bjorn3/browser_wasi_shim";
// @ts-ignore
import { __wbg_set_wasm } from "../dist/perspective_js_bg.js";
// @ts-ignore
import * as perspective_js_bg from "../dist/perspective_js_bg.js";
// @ts-ignore
import wasmfile from "../dist/perspective_js_bg.wasm";
import * as psp from "../dist/perspective_js.js";
import { Req } from "./msg.js";

let stderr = new OpenFile(new File([]));
let stdout = new OpenFile(new File([]));

let args: string[] = [];
let env = ["RUST_BACKTRACE=full"];
let fds = [
    new OpenFile(new File([])), // stdin
    stdout, // stdout
    stderr, // stderr
];
let wasi = new WASI(args, env, fds);

let selfAny = self as any;
selfAny.stderr = stderr;
selfAny.stdout = stdout;

let initWasm = (async () => {
    const module = await WebAssembly.compileStreaming(fetch(wasmfile));

    let inst = await WebAssembly.instantiate(module, {
        wasi_snapshot_preview1: wasi.wasiImport,
        // @ts-ignore
        // "./perspective_js_bg.js": await import("./perspective_js_bg.js"),
        "./perspective_js_bg.js": perspective_js_bg,
        env: {
            _ZNSt3__25mutexD1Ev: function (...args: any[]) {
                // console.log("std::__2::mutex::~mutex()", args);
            },
            _ZNSt3__25mutex4lockEv: function (...args: any[]) {
                // console.log("std::__2::mutex::lock()", args);
            },
            _ZNSt3__25mutex6unlockEv: function (...args: any[]) {
                // console.log("std::__2::mutex::unlock()", args);
            },
            mmap: function (...args: any[]) {
                console.log("mmap()", args);
            },
            mremap: function (...args: any[]) {
                console.log("mremap()", args);
            },
            munmap: function (...args: any[]) {
                console.log("munmap()", args);
            },
            pthread_rwlock_destroy: function (...args: any[]) {
                console.log("pthread_rwlock_destroy()", args);
            },
            pthread_rwlock_init: function (...args: any[]) {
                console.log("pthread_rwlock_init()", args);
            },
            pthread_rwlock_rdlock: function (...args: any[]) {
                console.log("pthread_rwlock_rdlock()", args);
            },
            pthread_rwlock_unlock: function (...args: any[]) {
                console.log("pthread_rwlock_unlock()", args);
            },
            pthread_rwlock_wrlock: function (...args: any[]) {
                console.log("pthread_rwlock_wrlock()", args);
            },
            __cxa_allocate_exception: function (...args: any[]) {
                console.log("EXCEPTION", args);
            },
            __cxa_throw: function (...args: any[]) {
                console.log("THROW", args);
            },
        },
    });

    __wbg_set_wasm(inst.exports);

    wasi.initialize(inst as any);
})();

let memclient: psp.JsClient | undefined;
let transport: psp.JsTransport | undefined;
let task: number | undefined;

self.onmessage = async (msg: MessageEvent<Req>) => {
    try {
        await initWasm;
        switch (msg.data.type) {
            case "init":
                console.log("Initializing", msg);
                transport = psp.PerspectiveTransport.make();
                memclient =
                    psp.MemoryPerspectiveClient.makeWithTransport(transport);
                transport.onTx((data: Uint8Array) => {
                    console.log("Server sending", data);
                    self.postMessage(data.buffer);
                });
                // TODO: This is wrong
                task = setInterval(() => {
                    memclient?.process();
                }, 1000);
                break;
            case "data":
                console.log("Server received", msg.data.data);
                if (transport) {
                    transport.rx(new Uint8Array(msg.data.data));
                }
                break;
        }
    } catch (e) {
        console.error(e);
        throw e;
    }
};
