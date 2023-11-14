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

const windowAny = window as any;
windowAny.psp = psp;

const module = await WebAssembly.compileStreaming(fetch(wasmfile));

windowAny.wasi = wasi;
windowAny.module = module;
// windowAny.psp = psp;

windowAny.perspective_bg = psp;

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
windowAny.inst = inst;
windowAny.decoder = new TextDecoder("utf-8");

__wbg_set_wasm(inst.exports);

// (wasi.inst as any) = inst;
// let exitCode = wasi.start(inst as any);
let exitCode = wasi.initialize(inst as any);

// This should print "hello world (exit code: 0)"
// window.stdout = stdout;
// let out = new TextDecoder("utf-8").decode(stdout.file.data);
// console.log(`stdout: ${out}` + `\n(exit code: ${exitCode})`);
windowAny.stderr = stderr;
windowAny.stdout = stdout;

// let socket = new WebSocket("ws://localhost:3000/ws");

// let socketOpen = new Promise((resolve, reject) => {
//     socket.onopen = resolve;
//     socket.onerror = reject;
// });

// await socketOpen;

const transport = psp.PerspectiveTransport.make();
windowAny.transport = transport;

// socket.onmessage = (msg: MessageEvent<Blob>) => {
//     msg.data
//         .arrayBuffer()
//         .then((x) => new Uint8Array(x))
//         .then((x) => transport.recv(x));
// };

let client = await psp.RemotePerspectiveClient.make(transport);
windowAny.client = client;

// let table = await client.makeTable();
// windowAny.table = table;

let worker = new Worker("./worker.js");
worker.postMessage({ type: "init", transport });
transport.onTx((msg: Uint8Array) => {
    console.log("Client sending", msg);
    worker.postMessage({ type: "data", data: msg.buffer }, [msg.buffer]);
});
worker.onmessage = (msg: MessageEvent) => {
    console.log("Client received", msg);
    transport.rx(new Uint8Array(msg.data));
};
windowAny.worker = worker;

// windowAny.superstore = fetch("superstore.arrow")
windowAny.superstore = fetch("untitled.arrow")
    .then((x) => x.arrayBuffer())
    .then((x) => new Uint8Array(x));

///////////////////////////
// API Prototyping below //
///////////////////////////

// Notes:
// - Separate Proxy object routing from transport layer (here we use a "manager").
//   - Allows transport to be stupid and just shuffle bytes around.
// - _MUST_ work with tradition Python web frameworks (Tornado, Aiohttp, Starlet, FastAPI, etc.)

// import {Client} from "@finos/perspective";

// const socket = new WebSocket("myurl");

// const transport = new psp.JsTransport();

// socket.onmessage = (msg) => transport.send(msg.data);
// transport.onmessage(msg => socket.send(msg));

// const manager = new Client(transport);

// const table = await manager.table("skajldfkajdsakl");
// console.log(await table.size());
