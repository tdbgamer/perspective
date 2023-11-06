import { JsTransport } from "../dist/perspective_js.js";

export type Req =
    | { type: "init"; transport: JsTransport }
    | { type: "data"; data: ArrayBuffer };
