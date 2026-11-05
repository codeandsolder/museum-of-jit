"""Guard fixture and separate-workload benchmark parsing."""
import hashlib
import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("museum_bench", ROOT / "tools/bench.py")
assert SPEC is not None and SPEC.loader is not None
bench = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(bench)


class ChallengeSha256WorkloadTest(unittest.TestCase):
    def test_fixture_matches_independent_sha256(self):
        fixture = json.loads((ROOT / "workloads/challenge-sha256-v1/fixture.json").read_text())
        prefix = fixture["prefix_ascii"].encode()
        self.assertEqual(len(prefix), 42)
        self.assertEqual(fixture["expected_check"], hashlib.sha256(prefix + b"100000000").hexdigest()[:16])
        for case in fixture["oracle_cases"]:
            self.assertEqual(
                hashlib.sha256(prefix + str(case["nonce"]).encode()).hexdigest(),
                case["sha256"],
            )

    def test_parser_cannot_pass_wrong_workload_or_probe(self):
        output = "ENGINE dotnet MODE portable VECTOR_LANES 8\nCHECK b4b920f70c0de8df\nRATE 123.45 sink=0123456789abcdef\n"
        parsed = bench.parse_output(output, "b4b920f70c0de8df")
        self.assertEqual(parsed["samples"][0]["sink"], "0123456789abcdef")
        self.assertEqual(parsed["check"], "b4b920f70c0de8df")
        self.assertEqual(
            bench.implementation_evidence("dotnet-challenge-portable", output),
            {"simd_lanes": 8, "backend": "portable"},
        )
        with self.assertRaises(ValueError):
            bench.parse_output(output)
        with self.assertRaises(ValueError):
            bench.implementation_evidence("dotnet-challenge-scalar", output)


if __name__ == "__main__":
    unittest.main()
