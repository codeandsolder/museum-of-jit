"""Strict identity checks for the canonical LLVM O0-O3 JIT adapters.

Run: uv run --no-project --python 3.13 python -m unittest discover -s tools/tests
This guards the benchmark's evidence layer, not HashWX algorithm correctness.
The independent 100k/627 Wasm differential tests cover the latter.
"""

from __future__ import annotations

import argparse
import importlib.util
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

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
                output = (
                    f"ENGINE llvm-orc VERSION 21.1.8\nMODE jit-batch\nIR_OPT {policy}\n"
                )
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
                    with (
                        self.subTest(malformed=malformed),
                        self.assertRaises(ValueError),
                    ):
                        bench.implementation_evidence(impl, malformed)


class BenchmarkArgumentTests(unittest.TestCase):
    def test_count_and_timeout_bounds(self) -> None:
        self.assertEqual(bench.parse_count("0", 0), 0)
        self.assertEqual(bench.parse_count(str(bench.MAX_COUNT)), bench.MAX_COUNT)
        for value in (
            "0",
            "-1",
            "+1",
            "1.0",
            "1e2",
            " 1",
            "\uff11\uff12",
            str(bench.MAX_COUNT + 1),
        ):
            with (
                self.subTest(value=value),
                self.assertRaises(argparse.ArgumentTypeError),
            ):
                bench.parse_count(value)
        for value in ("0", "-1", "nan", "inf", "-inf"):
            with self.assertRaises(argparse.ArgumentTypeError):
                bench.parse_timeout(value)

    def test_reject_missing_wrong_duplicate_and_unknown_dimensions(self) -> None:
        plan = bench.BenchmarkPlan(0, 128, 2, 1.0)
        valid = "BENCHMARK_DIMENSIONS v1 0 128 2\n"
        bench.validate_dimensions(valid, plan)
        for output in (
            "",
            valid.replace("128", "127"),
            valid + valid,
            valid.replace("v1", "v2"),
        ):
            with self.assertRaises(ValueError):
                bench.validate_dimensions(output, plan)

    @unittest.skipUnless(os.name == "posix", "POSIX process-group ownership")
    def test_timeout_owns_descendants_and_scratch(self) -> None:
        # Spawn a real child which survives the shell unless the whole group
        # is cleaned up. File-backed output must preserve the diagnostics.
        argv = [
            "/bin/sh",
            "-c",
            'mkdir "$TMPDIR/owned"; sleep 60 & printf "%s\\n%s\\n" "$!" "$TMPDIR"; wait',
        ]
        with self.assertRaises(subprocess.TimeoutExpired) as error:
            bench.run_command(argv, None, 0.5)
        child, scratch = error.exception.output.strip().splitlines()
        self.assertFalse(Path(scratch).exists())
        # Linux may leave a briefly unreaped zombie; it cannot execute or
        # write anything. Check actual liveness rather than just /proc.
        if Path("/proc").exists():
            for _ in range(100):
                try:
                    stat = (
                        Path(f"/proc/{child}/stat")
                        .read_text()
                        .rsplit(")", 1)[1]
                        .split()
                    )
                except FileNotFoundError:
                    break
                if stat[0] == "Z":
                    break
                time.sleep(0.01)
            else:
                self.fail("timed-out runner child is still executing")

    def test_explicit_dimensions_are_passed_and_recorded(self) -> None:
        with tempfile.TemporaryDirectory() as path:
            root = Path(path)
            runner = root / "runner"
            runner.write_text(
                "#!/bin/sh\n"
                'test "$#" = 3 && test "$1" = 0 && test "$2" = 128 && test "$3" = 2 || exit 64\n'
                "printf 'BENCHMARK_DIMENSIONS v1 0 128 2\\nCHECK fd5d450b767035c5\\nRATE 10.000 sink=0000000000000000\\nRATE 11.000 sink=0000000000000000\\n'\n"
            )
            runner.chmod(0o755)
            spec = {
                "runner": "runner",
                "mode": "fixture",
                "source": ".",
                "runtime": "fixture",
            }
            env = dict(
                os.environ,
                MUSEUM_WARMUP="99",
                MUSEUM_ITERATIONS="999",
                MUSEUM_SAMPLES="7",
            )
            with (
                patch.object(bench, "ROOT", root),
                patch.object(bench, "runtime_version", return_value="fixture"),
            ):
                result = bench.run_one(
                    "fixture",
                    spec,
                    env,
                    {},
                    "dummy",
                    bench.BenchmarkPlan(0, 128, 2, 5.0),
                )
            self.assertEqual(result["status"], "success")
            self.assertEqual(result["runtime"]["command"], ["runner", "0", "128", "2"])
            self.assertEqual(result["warmup_iterations"], 0)
            self.assertEqual(result["sample_iterations"], 128)
            self.assertEqual(result["sample_count"], 2)
            self.assertEqual(len(result["samples"]), 2)


if __name__ == "__main__":
    unittest.main()
