import json
from pathlib import Path

MAX_BATCH_LEN = 20000 - 500


def exportbatch(batch):
    v = json.dumps(dict(batch))
    quoted = json.dumps(v)
    return "{{ $data := " + quoted + " }}"


data = json.load(open("out.json"))
entries = list(data.items())
batch_start = 0

batches = []
while batch_start < len(entries):
    lo, hi = batch_start + 1, len(entries)
    while lo < hi:
        mid = (lo + hi + 1) // 2
        export = exportbatch(entries[batch_start:mid])
        if len(export) <= MAX_BATCH_LEN:
            lo = mid
        else:
            hi = mid - 1

    batch_end = lo
    batches.append(exportbatch(entries[batch_start:batch_end]))

    batch_start = batch_end

Path("yagdata").mkdir(exist_ok=True)
for n, out in enumerate(batches):
    with open(f"yagdata/{n}.txt", "w") as f:
        f.write(out)
