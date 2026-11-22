import fs from "node:fs";

const EXPECTED = 0xfd5d450b767035c5n;
const MASK = (1n << 64n) - 1n;

function count(raw, minimum = 1) {
  const value = Number(raw);
  if (!/^[0-9]+$/.test(raw) || !Number.isSafeInteger(value) || value < minimum || value > 2147483647) {
    throw new Error(`invalid count: ${raw}`);
  }
  return value;
}

function u64(value) {
  return BigInt.asUintN(64, value);
}

function hex64(value) {
  return u64(value).toString(16).padStart(16, "0");
}

const path = process.argv[2];
if (process.argv.length !== 6) throw new Error("usage: node v8.mjs module.wasm WARMUP ITERATIONS SAMPLES");
const warmup = count(process.argv[3], 0);
const iterations = count(process.argv[4]);
const samples = count(process.argv[5]);
const bytes = fs.readFileSync(path);

let t0 = process.hrtime.bigint();
const module = new WebAssembly.Module(bytes);
const instance = new WebAssembly.Instance(module, {});
const buildUs = Number(process.hrtime.bigint() - t0) / 1e3;
const { hash, bench } = instance.exports;

t0 = process.hrtime.bigint();
const check = u64(hash(123456n));
const firstUs = Number(process.hrtime.bigint() - t0) / 1e3;
if (check !== EXPECTED) throw new Error(`wrong result: ${hex64(check)} != ${hex64(EXPECTED)}`);

bench(0n, BigInt(warmup));
console.log(`ENGINE v8 NODE ${process.version} V8 ${process.versions.v8}`);
console.log(`BUILD_US ${buildUs.toFixed(3)} FIRST_US ${firstUs.toFixed(3)} CODE ${bytes.length} CHECK ${hex64(check)}`);
for (let sample = 0; sample < samples; sample++) {
  t0 = process.hrtime.bigint();
  const sink = u64(bench(0n, BigInt(iterations)));
  const seconds = Number(process.hrtime.bigint() - t0) / 1e9;
  console.log(`RATE ${(iterations / seconds).toFixed(3)} sink=${hex64(sink)}`);
}
