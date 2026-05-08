# idot test suite — runs all .id files in examples/ through the full pipeline.

import sys
import subprocess
from pathlib import Path

ROOT = Path(__file__).parent.parent
COMPILER = ROOT / "idotc.py"

PASS = "\033[32m✓\033[0m"
FAIL = "\033[31m✗\033[0m"

def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)

def test_spec_loads():
    r = run([sys.executable, str(COMPILER), "--spec"])
    ok = r.returncode == 0 and "idot" in r.stdout
    print(f"  {PASS if ok else FAIL} spec loads")
    return ok

def test_file(path: Path):
    # lex
    r = run([sys.executable, str(COMPILER), str(path), "--tokens"])
    if r.returncode != 0:
        print(f"  {FAIL} {path.name}: lex failed\n    {r.stderr.strip()}")
        return False

    # ast
    r = run([sys.executable, str(COMPILER), str(path), "--ast"])
    if r.returncode != 0:
        print(f"  {FAIL} {path.name}: parse failed\n    {r.stderr.strip()}")
        return False

    # emit C
    r = run([sys.executable, str(COMPILER), str(path), "--emit-c"])
    if r.returncode != 0:
        print(f"  {FAIL} {path.name}: codegen failed\n    {r.stderr.strip()}")
        return False

    # compile + run
    r = run([sys.executable, str(COMPILER), str(path)])
    if r.returncode != 0:
        print(f"  {FAIL} {path.name}: compile failed\n    {r.stderr.strip()}")
        return False

    # execute
    binary = path.with_suffix("")
    if binary.exists():
        r = run([str(binary)])
        if r.returncode != 0:
            print(f"  {FAIL} {path.name}: runtime error\n    {r.stderr.strip()}")
            return False
        print(f"  {PASS} {path.name}  output: {r.stdout.strip()[:60]}")
    else:
        print(f"  {PASS} {path.name}  (no binary produced)")
    return True


def main():
    print("=== idot test suite ===\n")
    results = []
    results.append(test_spec_loads())

    examples = sorted((ROOT / "examples").glob("*.id"))
    for ex in examples:
        results.append(test_file(ex))

    total = len(results)
    passed = sum(results)
    print(f"\n{'='*30}")
    print(f"  {passed}/{total} passed")
    sys.exit(0 if passed == total else 1)

if __name__ == "__main__":
    main()