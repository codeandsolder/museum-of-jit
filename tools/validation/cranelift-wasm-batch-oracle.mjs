// Compare the exact native Cranelift in-JIT batch-loop semantics with the
// independently compiled canonical WebAssembly bench(start,count) export.
import fs from 'node:fs';
const [modulePath, casesPath, outputPath] = process.argv.slice(2);
if (!modulePath || !casesPath || !outputPath || process.argv.length !== 5) {
  throw new Error('usage: node cranelift-wasm-batch-oracle.mjs MODULE.wasm CASES.hex OUTPUT.hex');
}
const module = new WebAssembly.Module(fs.readFileSync(modulePath));
const {bench} = new WebAssembly.Instance(module, {}).exports;
const cases = fs.readFileSync(casesPath, 'utf8').trimEnd().split('\n');
const output = cases.map((line, index) => {
  const fields = line.trim().split(/\s+/);
  if (fields.length !== 2 || fields.some(field => !/^[0-9a-fA-F]{16}$/.test(field))) {
    throw new Error(`invalid start/count case at line ${index + 1}`);
  }
  const result = bench(BigInt(`0x${fields[0]}`), BigInt(`0x${fields[1]}`));
  return BigInt.asUintN(64, result).toString(16).padStart(16, '0');
});
fs.writeFileSync(outputPath, `${output.join('\n')}\n`);
