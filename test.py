"""
This tests the solver for all possible starts and multiple board sizes (5-20)
"""
from subprocess import Popen, DEVNULL, TimeoutExpired
import math
from typing import List

for SIZE in range(5, 21):
    print(f"size: {SIZE}")

    procs: List[Popen] = []
    starts = []
    for i in range(math.ceil(SIZE/2)):
        for j in range(math.ceil(SIZE/2)):
            if i <= j:
                procs.append(Popen(["./target/debug/tenxten", str(i+1),
                                    str(j+1), "--board-size", str(SIZE), "--no-animation"], stdout=DEVNULL))
                starts.append((i+1, j+1))

    success = 0
    failed = []
    for p, s in zip(procs, starts):
        try:
            p.wait(timeout=10)
            success += 1
        except TimeoutExpired:
            failed.append(s)
        finally:
            print(
                f"\rsuccess: {success}/{len(procs)}, failed: {len(failed)}/{len(procs)}", end="")
            p.kill()
    print()
    if len(failed) > 0:
        print("FAILED:")
        print("\n".join([f"{i} {j}" for i, j in failed]))
    print("-"*10)
