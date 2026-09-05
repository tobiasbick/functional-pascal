"""Exercise the POSIX distribution script with isolated command fixtures."""

import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile


def check_distribution(shell: str, source: Path) -> None:
    script = source.read_text(encoding="utf-8")
    for failure in ("build", "stage", "copy", "chmod", "success", "outside"):
        with tempfile.TemporaryDirectory(prefix="fpas dist test ") as temporary:
            root = Path(temporary)
            project = root / "project"
            commands = root / "commands"
            release = project / "target" / "release"
            release.mkdir(parents=True)
            commands.mkdir()
            (project / "bin").mkdir()
            (project / "dist.sh").write_text(script, encoding="utf-8", newline="\n")
            for name in ("fpas", "fpas-runner"):
                (release / name).write_text("new", encoding="utf-8")
                (project / "bin" / name).write_text("old", encoding="utf-8")
            bodies = {
                "cargo": '''printf '%s\\n' "$1" >> "$TEST_LOG"
test -d target/release || exit 43
case "$TEST_FAILURE:$1" in build:build|stage:run) exit 42 ;; esac
''',
                "cp": '''test "$TEST_FAILURE" != copy || exit 42
exec /bin/cp "$@"
''',
                "chmod": '''test "$TEST_FAILURE" != chmod || exit 42
exec /bin/chmod "$@"
''',
            }
            for name, body in bodies.items():
                command = commands / name
                command.write_text("#!/bin/sh\n" + body, encoding="utf-8", newline="\n")
                command.chmod(0o755)
            log = root / "cargo.log"
            # Let the shell extend its native PATH (Windows and POSIX use different separators).
            result = subprocess.run(
                [shell, "-c", 'fixture_commands=$(cd "$1" && pwd) || exit; PATH="$fixture_commands:$PATH"; export PATH TEST_FAILURE="$2" TEST_LOG="$3"; sh "$4"',
                 "dist-test", commands.as_posix(), failure, log.as_posix(),
                 (project / "dist.sh").as_posix()],
                cwd=root if failure == "outside" else project,
                capture_output=True,
                text=True,
            )
            success = failure in ("success", "outside")
            assert (result.returncode == 0) == success, (failure, result.stdout, result.stderr)
            assert ("Built:" in result.stdout) == success, (failure, result.stdout)
            calls = log.read_text(encoding="utf-8").splitlines()
            assert calls == (["build"] if failure == "build" else ["build", "run"]), calls
            expected = "old" if failure in ("build", "stage", "copy") else "new"
            for name in ("fpas", "fpas-runner"):
                assert (project / "bin" / name).read_text(encoding="utf-8") == expected, failure
            assert not (root / "bin").exists(), "Distribution was written outside the project"
            print(f"PASS: {failure}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--shell", default=shutil.which("sh"))
    parser.add_argument("--source", type=Path, default=Path(__file__).resolve().parents[2] / "dist.sh")
    args = parser.parse_args()
    if not args.shell:
        parser.error("Pass --shell with a POSIX shell executable")
    check_distribution(args.shell, args.source)
