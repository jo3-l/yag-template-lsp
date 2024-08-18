import re
import subprocess
from pathlib import Path
from typing import Set

bundled_funcs: Set[str] = set()
for filename in Path.glob(Path(__file__).parent.parent / "bundled-defs", "*.ydef"):
    with open(filename) as f:
        for func in re.findall(r"^func\s+(\w+)", f.read(), re.MULTILINE):
            bundled_funcs.add(func)

lytfs_result = subprocess.run(["lytfs"], stdout=subprocess.PIPE)
latest_funcs = set(lytfs_result.stdout.decode("utf-8").splitlines())

new_funcs = sorted(latest_funcs - bundled_funcs)
diff = "\n".join(f"+ {f}" for f in new_funcs)
print(diff)
