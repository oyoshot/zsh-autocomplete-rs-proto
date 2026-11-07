#!/usr/bin/env python3
"""Compare already-built Cargo benchmarks on one runner in base/PR/PR/base order."""

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import statistics
import subprocess
import time
from pathlib import Path

BENCHES = ("fuzzy_bench", "text_processing_bench")
ORDER = ("base", "candidate", "candidate", "base")
BENCH_LINE = re.compile(
    r"^test (.+?) \.\.\. bench:\s+([\d,]+) ns/iter \(\+/- ([\d,]+)\)$"
)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def record_environment(output):
    output.mkdir(parents=True, exist_ok=True)
    data = {
        "platform": platform.platform(),
        "cpu_count": os.cpu_count(),
        "cpu_affinity": sorted(os.sched_getaffinity(0))
        if hasattr(os, "sched_getaffinity")
        else None,
        "build_flags": {
            key: os.environ.get(key, "")
            for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
        },
        "runner": {
            key: os.environ.get(key, "")
            for key in (
                "GITHUB_RUN_ID",
                "GITHUB_RUN_ATTEMPT",
                "RUNNER_NAME",
                "RUNNER_OS",
                "RUNNER_ARCH",
                "ImageOS",
                "ImageVersion",
            )
        },
    }
    for name, command in (
        ("cpu", ["lscpu"]),
        ("rustc", ["rustc", "-Vv"]),
        ("cargo", ["cargo", "-V"]),
    ):
        result = subprocess.run(command, capture_output=True, text=True, check=True)
        data[name] = result.stdout.strip()
    write_json(output / "environment.json", data)
    print(json.dumps(data, indent=2), flush=True)


def parse_results(text):
    results = {}
    for line in text.splitlines():
        line = line.strip()
        match = BENCH_LINE.fullmatch(line)
        if not match:
            if line.startswith("test ") and "bench:" in line:
                raise ValueError(f"Malformed benchmark result: {line}")
            continue
        name, value, spread = match.groups()
        value, spread = int(value.replace(",", "")), int(spread.replace(",", ""))
        if name in results or value <= 0:
            raise ValueError(f"Duplicate or non-positive benchmark: {name}")
        results[name] = {"value": value, "spread": spread}
    if not results:
        raise ValueError("Benchmark command produced no measurements")
    return results


def aggregate(runs):
    if not runs or not runs[0]:
        raise ValueError("Cannot aggregate empty measurements")
    names = set(runs[0])
    if any(set(run) != names for run in runs):
        raise ValueError(
            "Benchmark names changed between repetitions of the same revision"
        )
    entries = []
    for name in sorted(names):
        values = [run[name]["value"] for run in runs]
        entries.append(
            {
                "name": name,
                "unit": "ns/iter",
                "value": statistics.median(values),
                "range": f"run estimates {min(values)}–{max(values)} ns/iter",
                "extra": "Within-run spreads: "
                + ", ".join(str(run[name]["spread"]) for run in runs),
            }
        )
    return entries


def run_logged(command, cwd, env, path):
    with (
        path.open("w") as log,
        subprocess.Popen(
            command,
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        ) as process,
    ):
        for line in process.stdout:
            log.write(line)
            print(line, end="", flush=True)
        if process.wait() != 0:
            raise RuntimeError(f"Command failed; see {path}")


def build(checkout, target, destination, benches, output):
    destination.mkdir(parents=True)
    command = ["cargo", "bench", "--locked", "--no-run", "--message-format=json"]
    for bench in benches:
        command.extend(["--bench", bench])
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    # Cargo JSON preserves the executable path even for a cached build.
    result = subprocess.run(
        command, cwd=checkout, env=env, capture_output=True, text=True, check=False
    )
    (output / f"build-{destination.name}.jsonl").write_text(result.stdout)
    (output / f"build-{destination.name}.log").write_text(result.stderr)
    print(result.stderr, end="", flush=True)
    if result.returncode:
        raise RuntimeError(f"Benchmark build failed in {checkout}; see build logs")
    executables = {}
    for line in result.stdout.splitlines():
        artifact = json.loads(line)
        if artifact.get("reason") == "compiler-artifact" and artifact.get("executable"):
            name = artifact["target"]["name"]
            if "bench" in artifact["target"]["kind"] and name in benches:
                copied = destination / name
                shutil.copy2(artifact["executable"], copied)
                executables[name] = copied
    if set(executables) != set(benches):
        raise ValueError(f"Missing benchmark executables in {checkout}")
    return executables


def revision(checkout, repository_url):
    result = (
        subprocess.check_output(
            ["git", "show", "-s", "--format=%H%x00%aI%x00%an%x00%ae%x00%s", "HEAD"],
            cwd=checkout,
            text=True,
        )
        .rstrip("\n")
        .split("\0")
    )
    sha, timestamp, name, email, subject = result
    commit = {
        "id": sha,
        "timestamp": timestamp,
        "message": subject,
        "url": f"{repository_url}/commit/{sha}",
        "author": {"name": name, "email": email},
        "committer": {"name": name, "email": email},
    }
    return {
        "commit": commit,
        "lockfile_sha256": hashlib.sha256(
            (checkout / "Cargo.lock").read_bytes()
        ).hexdigest(),
    }


def compare(args):
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        raise ValueError(
            "Use a new output directory so old measurements cannot be reused"
        )
    record_environment(output)
    checkouts = {"base": args.base.resolve(), "candidate": args.candidate.resolve()}
    metadata = {
        label: revision(path, os.environ.get("REPOSITORY_URL", "local"))
        for label, path in checkouts.items()
    }
    method = {
        "order": ORDER,
        "benches": args.bench,
        "filter": args.filter,
        "warmup_seconds": args.warmup,
        "measurement_seconds": args.measurement,
        "sample_size": args.samples,
    }
    write_json(output / "method.json", {"revisions": metadata, **method})
    # Complete both builds before timing. Snapshot binaries so a shared target
    # directory cannot replace a baseline executable while building the PR.
    binaries = {
        label: build(
            path,
            args.target.resolve(),
            output.parent / (output.name + "-binaries") / label,
            args.bench,
            output,
        )
        for label, path in checkouts.items()
    }
    runs = {label: [] for label in checkouts}
    for index, label in enumerate(ORDER):
        measurements = {}
        for bench in args.bench:
            log_path = output / f"{index}-{label}-{bench}.log"
            env = dict(
                os.environ,
                CRITERION_HOME=str(output / "criterion" / str(index) / bench),
            )
            command = [
                str(binaries[label][bench]),
                "--bench",
                "--warm-up-time",
                str(args.warmup),
                "--measurement-time",
                str(args.measurement),
                "--sample-size",
                str(args.samples),
                "--noplot",
                "--output-format",
                "bencher",
            ]
            if args.filter:
                command.append(args.filter)
            run_logged(command, checkouts[label], env, log_path)
            result = parse_results(log_path.read_text())
            if measurements.keys() & result.keys():
                raise ValueError("Duplicate benchmark names across executables")
            measurements.update(result)
        runs[label].append(measurements)
    aggregates = {label: aggregate(values) for label, values in runs.items()}
    for label, entries in aggregates.items():
        write_json(output / f"{label}.json", entries)
    # github-action-benchmark's external data schema. This file contains only
    # the baseline measured above, never results from another runner.
    now = int(time.time() * 1000)
    write_json(
        output / "baseline.json",
        {
            "lastUpdate": now,
            "repoUrl": os.environ.get("REPOSITORY_URL", "local"),
            "entries": {
                "Benchmark": [
                    {
                        "commit": metadata["base"]["commit"],
                        "date": now,
                        "tool": "customSmallerIsBetter",
                        "benches": aggregates["base"],
                    }
                ]
            },
        },
    )
    base_names, candidate_names = (set(run[0]) for run in runs.values())
    write_json(
        output / "results.json",
        {
            "revisions": metadata,
            "method": method,
            "runs": runs,
            "added": sorted(candidate_names - base_names),
            "removed": sorted(base_names - candidate_names),
        },
    )
    summary = (
        f"\n### Same-runner benchmark comparison\n\n"
        f"Base: `{metadata['base']['commit']['id']}`; candidate: `{metadata['candidate']['commit']['id']}`.\n\n"
        f"Order: base → candidate → candidate → base. Each run: {args.samples} Criterion samples, "
        f"{args.warmup}s warmup, {args.measurement}s measurement per benchmark. "
        "Values are medians of the two run estimates; ranges show their minimum and maximum. "
        "New or removed benchmarks have no comparable ratio. "
        "CPU/toolchain details, raw logs, individual estimates, and Criterion data are in the benchmark-evidence artifact. "
        "This reduces host and time-order bias; it does not remove all shared-VM noise.\n"
    )
    print(summary)
    (output / "summary.md").write_text(summary)
    if summary_path := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(summary_path, "a") as stream:
            stream.write(summary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    environment_parser = subcommands.add_parser("environment")
    environment_parser.add_argument("output", type=Path)
    comparison = subcommands.add_parser("compare")
    for name in ("base", "candidate", "target", "output"):
        comparison.add_argument(f"--{name}", type=Path, required=True)
    comparison.add_argument("--bench", action="append", choices=BENCHES)
    comparison.add_argument("--filter")
    comparison.add_argument("--warmup", type=float, default=1)
    comparison.add_argument("--measurement", type=float, default=2)
    comparison.add_argument("--samples", type=int, default=30)
    args = parser.parse_args()
    if args.command == "environment":
        record_environment(args.output)
    else:
        if args.warmup <= 0 or args.measurement <= 0 or args.samples < 10:
            parser.error(
                "Positive timing durations and at least 10 samples are required"
            )
        args.bench = args.bench or list(BENCHES)
        compare(args)


if __name__ == "__main__":
    main()
