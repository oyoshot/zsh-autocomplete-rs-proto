import argparse
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import paired_benchmarks as paired


class Measurements(unittest.TestCase):
    def test_parses_real_bencher_output_with_separators_and_noise(self):
        values = paired.parse_results(
            "Gnuplot not found, using plotters backend\n\n"
            "test filter_scaling/10000 ... bench:   1,373,295 ns/iter (+/- 48,801)\n"
        )
        self.assertEqual(
            values, {"filter_scaling/10000": {"value": 1373295, "spread": 48801}}
        )

    def test_rejects_empty_duplicate_zero_and_malformed_measurements(self):
        for text in (
            "Testing filter_scaling/10000\nSuccess\n",
            "test sample ... bench: 0 ns/iter (+/- 0)\n",
            "test sample ... bench: 42 ns/iter (+/- 1)\n" * 2,
            "test sample ... bench: invalid ns/iter (+/- 1)\n",
        ):
            with self.subTest(text=text), self.assertRaises(ValueError):
                paired.parse_results(text)

    def test_aggregates_estimates_and_preserves_variability(self):
        result = paired.aggregate(
            [
                {"sample": {"value": 100, "spread": 2}},
                {"sample": {"value": 105, "spread": 3}},
            ]
        )[0]
        self.assertEqual(result["value"], 102.5)
        self.assertEqual(result["unit"], "ns/iter")
        self.assertIn("100–105", result["range"])
        self.assertIn("2, 3", result["extra"])

    def test_missing_repetitions_or_changing_cases_cannot_silently_compare(self):
        for runs in ([], [{}], [{"sample": {"value": 1}}, {}]):
            with self.subTest(runs=runs), self.assertRaises(ValueError):
                paired.aggregate(runs)


class Pipeline(unittest.TestCase):
    def test_failed_command_cannot_produce_a_successful_partial_report(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "output.log"
            with self.assertRaises(RuntimeError):
                paired.run_logged(
                    [
                        sys.executable,
                        "-c",
                        "print('partial result'); raise SystemExit(7)",
                    ],
                    temp,
                    os.environ.copy(),
                    path,
                )
            self.assertIn("partial result", path.read_text())

    def test_baseline_is_fresh_local_measurement_and_builds_finish_before_timing(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = argparse.Namespace(
                base=root / "base",
                candidate=root / "candidate",
                target=root / "target",
                output=root / "evidence",
                bench=["fuzzy_bench"],
                filter=None,
                warmup=1,
                measurement=2,
                samples=30,
            )
            stages = []

            def build(checkout, target, destination, benches, output):
                stages.append("build-" + checkout.name)
                return {"fuzzy_bench": checkout / "benchmark"}

            def revision(checkout, url):
                return {"commit": {"id": checkout.name}, "lockfile_sha256": "fixture"}

            def run(command, cwd, env, path):
                stages.append("run-" + cwd.name)
                value = 100 if cwd.name == "base" else 250
                path.write_text(
                    f"test common ... bench: {value} ns/iter (+/- 5)\n"
                    + (
                        "test added ... bench: 50 ns/iter (+/- 1)\n"
                        if cwd.name == "candidate"
                        else ""
                    )
                )

            with (
                patch.object(
                    paired, "record_environment", lambda output: output.mkdir()
                ),
                patch.object(paired, "build", build),
                patch.object(paired, "revision", revision),
                patch.object(paired, "run_logged", run),
                patch.dict(
                    os.environ, {"GITHUB_STEP_SUMMARY": str(root / "summary.md")}
                ),
            ):
                paired.compare(args)
            self.assertEqual(stages[:2], ["build-base", "build-candidate"])
            self.assertEqual(
                stages[2:], ["run-base", "run-candidate", "run-candidate", "run-base"]
            )
            baseline = json.loads((args.output / "baseline.json").read_text())[
                "entries"
            ]["Benchmark"]
            self.assertEqual(len(baseline), 1)
            self.assertEqual(baseline[0]["commit"]["id"], "base")
            self.assertEqual(baseline[0]["tool"], "customSmallerIsBetter")
            self.assertEqual(baseline[0]["benches"][0]["value"], 100)
            candidate = {
                row["name"]: row["value"]
                for row in json.loads((args.output / "candidate.json").read_text())
            }
            self.assertEqual(
                candidate["common"] / baseline[0]["benches"][0]["value"], 2.5
            )
            self.assertEqual(
                json.loads((args.output / "results.json").read_text())["added"],
                ["added"],
            )
            self.assertTrue((root / "summary.md").exists())
            with self.assertRaisesRegex(ValueError, "new output directory"):
                paired.compare(args)

    def test_cached_cargo_artifacts_are_snapshotted_before_shared_target_changes(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            executable = root / "compiled"
            executable.write_text("base binary")
            artifact = {
                "reason": "compiler-artifact",
                "fresh": True,
                "target": {"name": "fuzzy_bench", "kind": ["bench"]},
                "executable": str(executable),
            }
            result = subprocess.CompletedProcess(
                [], 0, stdout=json.dumps(artifact) + "\n", stderr=""
            )
            with patch.object(subprocess, "run", return_value=result):
                binaries = paired.build(
                    root, root / "target", root / "base-binaries", ["fuzzy_bench"], root
                )
            executable.write_text("candidate binary")
            self.assertEqual(binaries["fuzzy_bench"].read_text(), "base binary")


if __name__ == "__main__":
    unittest.main()
