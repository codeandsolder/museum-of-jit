"""Guard fixture and generated source independently of the Rust harness."""

import hashlib
import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class ChallengeSha256WorkloadTest(unittest.TestCase):
    def test_fixture_matches_independent_sha256(self) -> None:
        fixture = json.loads(
            (ROOT / "workloads/challenge-sha256-v1/fixture.json").read_text()
        )
        prefix = fixture["prefix_ascii"].encode()
        self.assertEqual(len(prefix), 42)
        self.assertEqual(
            fixture["expected_check"],
            hashlib.sha256(prefix + b"100000000").hexdigest()[:16],
        )
        for case in fixture["oracle_cases"]:
            self.assertEqual(
                hashlib.sha256(prefix + str(case["nonce"]).encode()).hexdigest(),
                case["sha256"],
            )

    def test_unrolled_source_in_sync(self) -> None:

        subprocess.run(
            [
                sys.executable,
                str(ROOT / "tools/generate_challenge_sha256_unrolled.py"),
                "--check",
            ],
            check=True,
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
