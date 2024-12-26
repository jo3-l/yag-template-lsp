import json
from argparse import ArgumentParser
from pathlib import Path
import sys
import shutil


def gen_yagdata(entries):
    v = json.dumps(dict(entries))
    quoted = json.dumps(v)
    return "{{ $data := " + quoted + " }}"


def chunk(entries, *, max_len):
    PROG_SIZE = 500

    chunks = []
    start = 0
    while start < len(entries):
        lo, hi = start + 1, len(entries)
        while lo < hi:
            mid = (lo + hi + 1) // 2
            data = gen_yagdata(entries[start:mid])
            if len(data) <= max_len - PROG_SIZE:
                lo = mid
            else:
                hi = mid - 1

        end = lo
        chunks.append(gen_yagdata(entries[start:end]))
        start = end
    return chunks


p = ArgumentParser(
    prog="splitdefs",
    description="Split definition JSON into chunks to import into YAGPDB",
)
p.add_argument("filename")
p.add_argument("-m", "--max-len", type=int, default=10_000)
p.add_argument("-o", "--output-dir", default="yagdata")

args = p.parse_args()
try:
    with open(args.filename) as f:
        data = json.load(f)
except FileNotFoundError:
    print(f"file {args.filename} not found")
    sys.exit(1)
except json.JSONDecodeError as e:
    print(f"file {args.filename} contains invalid JSON: {e}")
    sys.exit(1)

chunks = chunk(list(data.items()), max_len=args.max_len)

out_dir = Path(args.output_dir)
shutil.rmtree(out_dir, ignore_errors=True)
out_dir.mkdir(parents=True)

for n, data in enumerate(chunks):
    with open(out_dir / f"{n}.txt", "w") as f:
        f.write(data)

print(f"wrote {len(chunks)} chunks")
