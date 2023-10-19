import {
  WASI,
  File,
  OpenFile,
  PreopenDirectory,
} from "@bjorn3/browser_wasi_shim";
// @ts-ignore
import { __wbg_set_wasm } from "../dist/perspective_bg";
// @ts-ignore
import wasmfile from "../dist/perspective_bg.wasm";
import { make_table, get_col_dtype } from "../dist/perspective.js";

console.log('wasmfile', wasmfile);

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

const module = await WebAssembly.compileStreaming(
  // fetch("out/perspective_bg.wasm")
  fetch(wasmfile)
);

windowAny.wasi = wasi;
windowAny.module = module;
windowAny.get_col_dtype = get_col_dtype;

let inst = await WebAssembly.instantiate(module, {
  wasi_snapshot_preview1: wasi.wasiImport,
  env: {
    // std::__2::mutex::~mutex()
    // @ts-ignore
    "_ZNSt3__25mutexD1Ev": function (...args) {console.log('std::__2::mutex::~mutex()', args)},
    // @ts-ignore
    "_ZNSt3__25mutex4lockEv": function (...args) {console.log('std::__2::mutex::lock()', args)},
    // @ts-ignore
    "_ZNSt3__25mutex6unlockEv": function (...args) {console.log('std::__2::mutex::unlock()', args)},
    // @ts-ignore
    "mmap": function (...args) {console.log('mmap()', args)},
    // @ts-ignore
    "mremap": function (...args) {console.log('mremap()', args)},
    // @ts-ignore
    "munmap": function (...args) {console.log('munmap()', args)},
    // @ts-ignore
    "__cxa_allocate_exception": function (...args) {console.log('EXCEPTION', args)},
    // @ts-ignore
    "__cxa_throw": function (...args) {console.log('THROW', args)},
  }
});
windowAny.inst = inst;
windowAny.decoder = new TextDecoder("utf-8");

__wbg_set_wasm(inst.exports);

windowAny.make_table = make_table;

// (wasi.inst as any) = inst;
// let exitCode = wasi.start(inst as any);
let exitCode = wasi.initialize(inst as any);

// This should print "hello world (exit code: 0)"
// window.stdout = stdout;
// let out = new TextDecoder("utf-8").decode(stdout.file.data);
// console.log(`stdout: ${out}` + `\n(exit code: ${exitCode})`);
windowAny.stderr = stderr;
windowAny.stdout = stdout;

