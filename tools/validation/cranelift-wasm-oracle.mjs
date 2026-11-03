// Independent oracle: run the canonical emitted fixed HashWX WebAssembly hash.
// Inputs and outputs are one unsigned 64-bit hexadecimal word per line.
import fs from 'node:fs';

const [modulePath, saltsPath, outputsPath] = process.argv.slice(2);
if (!modulePath || !saltsPath || !outputsPath || process.argv.length !== 5) {
  throw new Error('usage: node cranelift-wasm-oracle.mjs MODULE.wasm SALTS.hex OUTPUTS.hex');
}
const wasm = fs.readFileSync(modulePath);
const { hash } = new WebAssembly.Instance(new WebAssembly.Module(wasm), {}).exports;
const salts = fs.readFileSync(saltsPath, 'utf8').trimEnd().split('\n');
const hashes = salts.map((salt, index) => {
  if (!/^[a-f0-9]{16}$/i.test(salt)) throw new Error(`bad salt at line ${index + 1}`);
  return BigInt.asUintN(64, hash(BigInt(`0x${salt}`))).toString(16).padStart(16, '0');
});
fs.writeFileSync(outputsPath, `${hashes.join('\n')}\n`);
