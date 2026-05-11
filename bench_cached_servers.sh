./ben#!/usr/bin/env bash
set -euo pipefail
requests=${1:-500}
concurrency=${2:-20}
./sitegen >/dev/null
wait_http(){ local u=$1; for i in $(seq 1 80); do curl -fsS "$u" >/dev/null 2>&1 && return 0; sleep 0.05; done; return 1; }
cleanup(){ [ -n "${TS_PID:-}" ] && kill "$TS_PID" 2>/dev/null || true; [ -n "${NODE_PID:-}" ] && kill "$NODE_PID" 2>/dev/null || true; }
trap cleanup EXIT

echo "Cached HTTP benchmark: no per-request filesystem reads"
echo "Requests: $requests, concurrency: $concurrency"
echo ""

tetherscript run --grant-fs examples/content_site examples/static_site_server_cached.tether >/tmp/ts-cached.log 2>&1 &
TS_PID=$!
wait_http http://127.0.0.1:8790/
echo "tetherscript cached:"
node examples/http_bench_client.js http://127.0.0.1:8790/ "$requests" "$concurrency"
kill "$TS_PID" 2>/dev/null || true; TS_PID=""; sleep 0.2

QUIET=1 PORT=8791 node examples/static_site_server_cached.js >/tmp/node-cached.log 2>&1 &
NODE_PID=$!
wait_http http://127.0.0.1:8791/
echo "node.js cached:"
node examples/http_bench_client.js http://127.0.0.1:8791/ "$requests" "$concurrency"
kill "$NODE_PID" 2>/dev/null || true; NODE_PID=""
