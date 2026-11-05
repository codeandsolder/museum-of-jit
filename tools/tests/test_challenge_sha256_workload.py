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

    def test_rust_provenance_requires_exact_binary_sha(self):
        sha = "a" * 64
        example = (
            "BINARY_SHA256 " + sha + "\n"
            "ENGINE rust-challenge MODE simd VECTOR_LANES 16\n"
            "CHECK b4b920f70c0de8df\n"
            "RATE 99999999.9 sink=00000000002dc6c0\n"
        )
        evidence = bench.implementation_evidence("rust-challenge-simd", example)
        self.assertEqual(evidence["binary_sha256"], sha)
        self.assertEqual(evidence["simd_lanes"], 16)
        with self.assertRaises(ValueError):
            bench.implementation_evidence(
                "rust-challenge-simd", example.replace("BINARY_SHA256 " + sha, "")
            )

    def test_match_backend_and_exact_work_count(self):
        output = (
            "ENGINE dotnet MODE match VECTOR_LANES 16\\n"
            "CHECK b4b920f70c0de8df\\n"
            "RATE 1000000.0 sink=0000000000000400\\n"
        )
        output = output.replace("\\n", "\n")
        evidence = bench.implementation_evidence("dotnet-challenge-match", output)
        self.assertEqual(evidence["backend"], "match")
        self.assertEqual(evidence["simd_lanes"], 16)
        parsed = bench.parse_output(output, "b4b920f70c0de8df")
        bench.validate_challenge_sink("dotnet-challenge-match", parsed, 1024)
        bench.validate_challenge_sink("rust-challenge-simd", parsed, 1024)
        with self.assertRaises(ValueError):
            bench.validate_challenge_sink("dotnet-challenge-match", parsed, 1025)
        with self.assertRaises(ValueError):
            bench.implementation_evidence("dotnet-challenge-portable", output)

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
