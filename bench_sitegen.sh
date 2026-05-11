#!/usr/bin/env bash
set -euo pipefail
runs=${1:-20}

ts_cmd=(tetherscript run --grant-fs examples/content_site examples/static_site_gen.tether)
node_cmd=(node examples/static_site_gen.js)

bench() {
  local name=$1
  shift
  local total=0
  local min=999999999
  local max=0
  local i=1
  while [ "$i" -le "$runs" ]; do
    rm -rf examples/content_site/dist
    mkdir -p examples/content_site/dist
    local start end dur
    start=$(python - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
    QUIET=1 "$@" >/dev/null
    end=$(python - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
    dur=$(( (end - start) / 1000000 ))
    total=$(( total + dur ))
    if [ "$dur" -lt "$min" ]; then min=$dur; fi
    if [ "$dur" -gt "$max" ]; then max=$dur; fi
    i=$((i + 1))
  done
  local avg=$(( total / runs ))
  printf '%-12s avg=%4sms min=%4sms max=%4sms runs=%s\n' "$name" "$avg" "$min" "$max" "$runs"
}

echo "Static site generator benchmark"
echo "Workload: read 3 markdown files, parse frontmatter, generate 3 pages + index + JSON + RSS"
echo ""
bench tetherscript "${ts_cmd[@]}"
bench node.js "${node_cmd[@]}"
