import psp from "@finos/perspective";
import * as fs from "fs";
import * as path from "path";

const arrow = fs.readFileSync(path.join(__dirname, "dist/superstore.arrow"));

for (let i = 0; i < 100; i++) {
    const foo = await psp.table(arrow);
    await foo.delete();
}

console.time("init");
const foo = await psp.table(arrow);
console.timeEnd("init");

console.log("table_size", await foo.size());

await foo.delete();
