import {
  WASI,
  File,
  OpenFile,
  PreopenDirectory,
} from "@bjorn3/browser_wasi_shim";
// @ts-ignore
import { __wbg_set_wasm } from "../dist/perspective_bg.js";
// @ts-ignore
import wasmfile from "../dist/perspective_bg.wasm";
import { make_table } from "../dist/perspective.js";
import * as psp from "../dist/perspective.js";

let stderr = new OpenFile(new File([]));
let stdout = new OpenFile(new File([]));

let args: string[] = [];
let env = ["RUST_BACKTRACE=full"];
let fds = [
  new OpenFile(new File([])), // stdin
  stdout, // stdout
  stderr, // stderr
];
let wasi = new WASI(args, env, fds, { debug: true });

const windowAny = window as any;
windowAny.psp = psp;
// windowAny.Table = Table;
windowAny.make_table = make_table;

const module = await WebAssembly.compileStreaming(
  // fetch("out/perspective_bg.wasm")
  fetch(wasmfile)
);

windowAny.wasi = wasi;
windowAny.module = module;
// windowAny.psp = psp;

windowAny.perspective_bg = psp;

let inst = await WebAssembly.instantiate(module, {
  wasi_snapshot_preview1: wasi.wasiImport,
    // @ts-ignore
  "./perspective_bg.js": {
    // @ts-ignore
    "__wbindgen_throw": function(...args) { console.log('throw', args); }
  },
  env: {
    "_ZNSt3__25mutexD1Ev": function (...args: any[]) {console.log('std::__2::mutex::~mutex()', args)},
    "_ZNSt3__25mutex4lockEv": function (...args: any[]) {console.log('std::__2::mutex::lock()', args)},
    "_ZNSt3__25mutex6unlockEv": function (...args: any[]) {console.log('std::__2::mutex::unlock()', args)},
    "mmap": function (...args: any[]) {console.log('mmap()', args)},
    "mremap": function (...args: any[]) {console.log('mremap()', args)},
    "munmap": function (...args: any[]) {console.log('munmap()', args)},
    "pthread_rwlock_destroy": function (...args: any[]) {console.log('pthread_rwlock_destroy()', args)},
    "pthread_rwlock_init": function (...args: any[]) {console.log('pthread_rwlock_init()', args)},
    "pthread_rwlock_rdlock": function (...args: any[]) {console.log('pthread_rwlock_rdlock()', args)},
    "pthread_rwlock_unlock": function (...args: any[]) {console.log('pthread_rwlock_unlock()', args)},
    "pthread_rwlock_wrlock": function (...args: any[]) {console.log('pthread_rwlock_wrlock()', args)},
    "__cxa_allocate_exception": function (...args: any[]) {console.log('EXCEPTION', args)},
    "__cxa_throw": function (...args: any[]) {console.log('THROW', args)},
  }
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

