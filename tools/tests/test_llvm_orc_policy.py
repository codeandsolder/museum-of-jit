#!/usr/bin/env python3
"""Strict identity checks for the canonical LLVM O0–O3 JIT adapters.

Run: uv run --no-project --python 3.13 python -m unittest discover -s tools/tests
This guards the benchmark's evidence layer, not HashWX algorithm correctness.
The independent 100k/627 Wasm differential tests cover the latter.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("museum_bench", ROOT / "tools/bench.py")
assert spec is not None and spec.loader is not None
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)


class PolicyAttestationTests(unittest.TestCase):
    def test_all_policies_and_exact_evidence(self) -> None:
        policies = {
            "llvm-orc-o0": "O0",
            "llvm-orc-o1": "O1",
            "llvm-orc": "O2",
            "llvm-orc-o3": "O3",
        }
        for impl, policy in policies.items():
            with self.subTest(impl=impl):
                output = f"ENGINE llvm-orc VERSION 21.1.8\nMODE jit-batch\nIR_OPT {policy}\n"
                self.assertEqual(
                    bench.implementation_evidence(impl, output),
                    {
                        "jit_batch_internal": True,
                        "llvm_ir_optimization": policy,
                        "llvm_codegen": "Aggressive",
                    },
                )
                incorrect = [
                    output.replace(f"IR_OPT {policy}", "IR_OPT O99"),
                    output.replace("MODE jit-batch", "MODE direct"),
                    output.replace("ENGINE llvm-orc", "ENGINE cranelift"),
                    output.replace(f"IR_OPT {policy}\n", ""),
                    output + f"IR_OPT {policy}\n",
                ]
                for malformed in incorrect:
                    with self.subTest(malformed=malformed):
                        with self.assertRaises(ValueError):
                            bench.implementation_evidence(impl, malformed)


if __name__ == "__main__":
    unittest.main()
