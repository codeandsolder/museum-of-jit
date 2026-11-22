"""Exercise the real builder's side-effect-free configuration boundary."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

BUILDER = Path(__file__).resolve().parents[1] / "build-cpython-jit.sh"


class CpythonBuildConfigTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="museum-build-config-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.runtime = self.root / "runtime"
        self.build = self.root / "build"
        self.sysroot = self.root / "sysroot"
        (self.sysroot / "usr/include").mkdir(parents=True)
        (self.sysroot / "usr/lib").mkdir(parents=True)

    def plan(
        self, *args: str, env: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                str(BUILDER),
                str(self.runtime),
                str(self.build),
                str(self.sysroot),
                "--plan",
                *args,
            ],
            text=True,
            capture_output=True,
            check=False,
            env=env,
            timeout=10,
        )

    def test_explicit_config_and_no_side_effects(self) -> None:
        # Removed project-specific environment controls cannot affect defaults.
        defaults = self.plan(
            env={
                **os.environ,
                "MUSEUM_CPYTHON_MAKE_JOBS": "99",
                "MUSEUM_SCCACHE_PROBE_PORT": "65535",
                "MUSEUM_CPYTHON_OBJDIR": str(self.root / "wrong-object"),
                "MUSEUM_CPYTHON_MAX_GLIBC": "2.1",
            }
        )
        self.assertEqual(defaults.returncode, 0, defaults.stderr)
        self.assertIn("make_jobs=4\n", defaults.stdout)
        self.assertIn(f"objdir={self.build}/obj\n", defaults.stdout)
        self.assertIn("sccache_probe_port=default\n", defaults.stdout)
        self.assertIn("max_glibc=unrestricted\n", defaults.stdout)
        configured = self.plan(
            "--object-dir",
            str(self.root / "external-object"),
            "--jobs",
            "7",
            "--sccache-probe-port",
            "4226",
            "--max-glibc",
            "2.41",
        )
        self.assertEqual(configured.returncode, 0, configured.stderr)
        self.assertIn("make_jobs=7\n", configured.stdout)
        self.assertIn("sccache_probe_port=4226\n", configured.stdout)
        self.assertIn("sccache_jit_port=4226\n", configured.stdout)
        self.assertIn("max_glibc=2.41\n", configured.stdout)
        self.assertIn(f"objdir={self.root}/external-object\n", configured.stdout)
        self.assertEqual(set(self.root.iterdir()), {self.sysroot})

    def test_invalid_arguments_are_rejected_before_acquisition(self) -> None:
        for args in [
            ("--unknown",),
            ("--jobs",),
            ("--jobs", "4", "--jobs", "4"),
            ("--jobs", "0"),
            ("--jobs", "999999999999999999999"),
            ("--sccache-jit-port", "65536"),
            ("--max-glibc", "glIBC_2.41"),
            ("--object-dir", str(self.sysroot / "objects")),
            ("--object-dir", str(self.build / "install")),
            ("--object-dir", str(self.root)),
        ]:
            with self.subTest(args=args):
                self.assertEqual(self.plan(*args).returncode, 64)
        self.assertEqual(set(self.root.iterdir()), {self.sysroot})

    def test_ownership_prevents_reclaiming_unrelated_directories(self) -> None:
        self.build.mkdir()
        unrelated = self.build / "unrelated-file"
        unrelated.write_text("retain")
        self.assertEqual(self.plan().returncode, 64)
        self.assertEqual(unrelated.read_text(), "retain")
        unrelated.unlink()
        (self.build / ".museum-cpython-owner").write_text(f"{self.build}\n")
        obj = self.root / "external-object"
        obj.mkdir()
        marker = obj / ".museum-cpython-owner"
        marker.write_text(f"{self.root}/other-build\n")
        self.assertEqual(self.plan("--object-dir", str(obj)).returncode, 64)
        marker.write_text(f"{self.build}\n")
        self.assertEqual(self.plan("--object-dir", str(obj)).returncode, 0)
        (self.build / "obj").symlink_to(obj, target_is_directory=True)
        self.assertEqual(self.plan().returncode, 64)


if __name__ == "__main__":
    unittest.main()
