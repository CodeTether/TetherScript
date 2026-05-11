#!/usr/bin/env bash
set -euo pipefail
requests=${1:-500}
concurrency=${2:-10}

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

cleanup() {
  if [ -n "${TS_PID:-}" ]; then kill "$TS_PID" 2>/dev/null || true; fi
  if [ -n "${NODE_PID:-}" ]; then kill "$NODE_PID" 2>/dev/null || true; fi
}
trap cleanup EXIT

echo "Static site HTTP server benchmark (keep-alive client)"
echo "Workload: serve generated index.html over localhost"
echo "Requests: $requests, concurrency: $concurrency"
echo ""

tetherscript run --grant-fs examples/content_site examples/static_site_server.tether >/tmp/tetherscript-site-server.log 2>&1 &
TS_PID=$!
wait_http http://127.0.0.1:8788/
echo "tetherscript:"
node examples/http_bench_client.js http://127.0.0.1:8788/ "$requests" "$concurrency"
kill "$TS_PID" 2>/dev/null || true
TS_PID=""
sleep 0.2

QUIET=1 PORT=8789 node examples/static_site_server.js >/tmp/node-site-server.log 2>&1 &
NODE_PID=$!
wait_http http://127.0.0.1:8789/
echo "node.js:"
node examples/http_bench_client.js http://127.0.0.1:8789/ "$requests" "$concurrency"
kill "$NODE_PID" 2>/dev/null || true
NODE_PID=""
