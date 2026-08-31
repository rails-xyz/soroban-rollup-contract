#!/usr/bin/env python3

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
PROJECT_DIR = SCRIPT_DIR
COMMAND = (
    # soroban-sdk 26 enables the `experimental_spec_shaking_v2` feature through
    # the stellar-* dependencies and refuses to build unless the build system
    # declares support for it. Certora builds with plain cargo, so declare it
    # here. It only affects contract-spec metadata, not the verified code.
    "SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2=1 "
    "RUSTFLAGS='-C link-arg=--allow-undefined' "
    "cargo build --target wasm32v1-none --release -p rollup-contract --features certora"
)
SOURCES = [
    "Cargo.toml",
    "contracts/rollup/Cargo.toml",
    "contracts/rollup/src/**/*.rs",
]
EXECUTABLES = "target/wasm32v1-none/release/rollup_contract.wasm"
VERBOSE = False


def log(msg: str) -> None:
    if VERBOSE:
        print(msg, file=sys.stderr)


def run_command(command: str, to_stdout: bool = False):
    log(f"Running '{command}'")
    try:
        if to_stdout:
            result = subprocess.run(command, shell=True, text=True, cwd=PROJECT_DIR)
            return None, None, result.returncode

        with tempfile.NamedTemporaryFile(
            delete=False, mode="w", prefix="certora_build_", suffix=".stdout"
        ) as stdout_file, tempfile.NamedTemporaryFile(
            delete=False, mode="w", prefix="certora_build_", suffix=".stderr"
        ) as stderr_file:
            result = subprocess.run(
                command,
                shell=True,
                stdout=stdout_file,
                stderr=stderr_file,
                text=True,
                cwd=PROJECT_DIR,
            )
            return stdout_file.name, stderr_file.name, result.returncode
    except Exception as exc:
        log(f"Error running command '{command}': {exc}")
        return None, None, -1


def write_output(output_data, output_file=None) -> None:
    if output_file:
        with open(output_file, "w", encoding="utf-8") as f:
            json.dump(output_data, f, indent=4)
    else:
        print(json.dumps(output_data, indent=4), file=sys.stdout)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compile the rollup contract and emit Certora build metadata."
    )
    parser.add_argument("-o", "--output", metavar="FILE", help="Write JSON output to a file.")
    parser.add_argument("--json", action="store_true", help="Print JSON output to stdout.")
    parser.add_argument("-l", "--log", action="store_true", help="Show build logs on stdout.")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose logging.")
    args = parser.parse_args()

    global VERBOSE
    VERBOSE = args.verbose

    stdout_log, stderr_log, return_code = run_command(COMMAND, args.log)
    output_data = {
        "project_directory": str(PROJECT_DIR),
        "sources": SOURCES,
        "executables": EXECUTABLES,
        "success": return_code == 0,
        "return_code": return_code,
        "log": {"stdout": stdout_log, "stderr": stderr_log},
    }

    if args.output:
        write_output(output_data, args.output)
    if args.json:
        write_output(output_data)

    sys.exit(0 if return_code == 0 else 1)


if __name__ == "__main__":
    main()
