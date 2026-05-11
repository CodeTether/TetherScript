#!/usr/bin/env bash
set -euo pipefail

requests=${1:-5000}
concurrency=${2:-50}
ts_bin=${TS_BIN:-./target/release/tetherscript}

mkdir -p examples/content_site/dist
"$ts_bin" run --grant-fs examples/content_site examples/static_site_gen.tether >/dev/null

wait_http() {
  local url=$1
  for _ in $(seq 1 80); do
    curl -fsS "$url" >/dev/null 2>&1 && return 0
    sleep 0.05
  done
  echo "server did not start: $url" >&2
  return 1
}

cleanup() {
  [ -n "${TS_PID:-}" ] && kill "$TS_PID" 2>/dev/null || true
  [ -n "${NODE_PID:-}" ] && kill "$NODE_PID" 2>/dev/null || true
}
trap cleanup EXIT

echo "Native static HTTP benchmark"
echo "Requests: $requests, concurrency: $concurrency"
echo ""

"$ts_bin" run --grant-fs examples/content_site examples/static_site_server_native.tether >/tmp/ts-static-native.log 2>&1 &
TS_PID=$!
wait_http http://127.0.0.1:8792/
echo "tetherscript native static:"
node examples/http_bench_client.js http://127.0.0.1:8792/ "$requests" "$concurrency"
kill "$TS_PID" 2>/dev/null || true
TS_PID=""
sleep 0.2

QUIET=1 PORT=8791 node examples/static_site_server_cached.js >/tmp/node-static-native.log 2>&1 &
NODE_PID=$!
wait_http http://127.0.0.1:8791/
echo "node.js cached:"
node examples/http_bench_client.js http://127.0.0.1:8791/ "$requests" "$concurrency"
kill "$NODE_PID" 2>/dev/null || true
NODE_PID=""
