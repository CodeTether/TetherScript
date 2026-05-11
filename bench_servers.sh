#!/usr/bin/env bash
set -euo pipefail
requests=${1:-200}

mkdir -p examples/content_site/dist
./sitegen >/dev/null

wait_http() {
  local url=$1
  local i=0
  until curl -fsS "$url" >/dev/null 2>&1; do
    i=$((i + 1))
    if [ "$i" -gt 80 ]; then
      echo "server did not start: $url" >&2
      return 1
    fi
    sleep 0.05
  done
}

bench_http() {
  local name=$1
  local url=$2
  local total=0
  local min=999999999
  local max=0
  local i=1
  while [ "$i" -le "$requests" ]; do
    local start end dur
    start=$(python - <<'PY'
import time
print(time.perf_counter_ns())
PY
)
    curl -fsS "$url" >/dev/null
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
  local avg=$(( total / requests ))
  printf '%-12s avg=%4sms min=%4sms max=%4sms reqs=%s\n' "$name" "$avg" "$min" "$max" "$requests"
}

cleanup() {
  if [ -n "${TS_PID:-}" ]; then kill "$TS_PID" 2>/dev/null || true; fi
  if [ -n "${NODE_PID:-}" ]; then kill "$NODE_PID" 2>/dev/null || true; fi
}
trap cleanup EXIT

echo "Static site HTTP server benchmark"
echo "Workload: serve generated index.html over localhost, sequential curl requests"
echo ""

# TetherScript server
tetherscript run --grant-fs examples/content_site examples/static_site_server.tether >/tmp/tetherscript-site-server.log 2>&1 &
TS_PID=$!
wait_http http://127.0.0.1:8788/
bench_http tetherscript http://127.0.0.1:8788/
kill "$TS_PID" 2>/dev/null || true
TS_PID=""
sleep 0.2

# Node server
QUIET=1 PORT=8789 node examples/static_site_server.js >/tmp/node-site-server.log 2>&1 &
NODE_PID=$!
wait_http http://127.0.0.1:8789/
bench_http node.js http://127.0.0.1:8789/
kill "$NODE_PID" 2>/dev/null || true
NODE_PID=""
