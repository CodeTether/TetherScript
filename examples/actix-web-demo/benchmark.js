#!/usr/bin/env node
const http = require('http');

const requests = Number(process.argv[2] || 3000);
const concurrency = Number(process.argv[3] || 32);
const rounds = Number(process.argv[4] || 3);
const origin = 'http://127.0.0.1:18081';
const routes = {
  rust: '/rust/country/USA',
  tetherscript: '/tether/country/USA',
};

function run(path, count) {
  return new Promise(resolve => {
    const agent = new http.Agent({ keepAlive: true, maxSockets: concurrency });
    const started = process.hrtime.bigint();
    let next = 0;
    let completed = 0;
    let inFlight = 0;
    let failed = 0;
    let responses = 0;
    const failureReasons = {};
    let bytes = 0;
    let latency = 0;
    let min = Number.POSITIVE_INFINITY;
    let max = 0;

    function finish() {
      const seconds = Number(process.hrtime.bigint() - started) / 1e9;
      agent.destroy();
      resolve({
        requests: count,
        concurrency,
        failed,
        failure_reasons: failureReasons,
        seconds: Number(seconds.toFixed(3)),
        attempt_rps: Number((count / seconds).toFixed(1)),
        success_rps: Number(((count - failed) / seconds).toFixed(1)),
        avg_ms: responses ? Number((latency / responses).toFixed(3)) : null,
        min_ms: responses ? Number(min.toFixed(3)) : null,
        max_ms: responses ? Number(max.toFixed(3)) : null,
        bytes,
      });
    }

    function pump() {
      while (inFlight < concurrency && next < count) {
        next++;
        inFlight++;
        const requestStarted = process.hrtime.bigint();
        const request = http.get(origin + path, { agent }, response => {
          response.on('data', chunk => { bytes += chunk.length; });
          response.on('end', () => {
            const ms = Number(process.hrtime.bigint() - requestStarted) / 1e6;
            latency += ms;
            responses++;
            min = Math.min(min, ms);
            max = Math.max(max, ms);
            if (response.statusCode !== 200) {
              failed++;
              const reason = `HTTP_${response.statusCode}`;
              failureReasons[reason] = (failureReasons[reason] || 0) + 1;
            }
            done();
          });
        });
        request.setTimeout(5000, () => request.destroy(new Error('ETIMEDOUT')));
        request.on('error', error => {
          failed++;
          const reason = error.code || error.message || 'UNKNOWN_SOCKET_ERROR';
          failureReasons[reason] = (failureReasons[reason] || 0) + 1;
          done();
        });
      }
    }

    function done() {
      completed++;
      inFlight--;
      if (completed === count) finish();
      else pump();
    }

    pump();
  });
}

function preflight(name, path) {
  return new Promise((resolve, reject) => {
    const request = http.get(origin + path, response => {
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => {
        const body = Buffer.concat(chunks).toString();
        if (response.statusCode !== 200) {
          reject(new Error(`${name} preflight returned HTTP ${response.statusCode}: ${body}`));
        } else {
          console.error(`preflight ok: ${name} ${origin}${path}`);
          resolve();
        }
      });
    });
    request.setTimeout(3000, () => request.destroy(new Error('ETIMEDOUT')));
    request.on('error', error => {
      reject(new Error(`${name} preflight failed: ${error.code || error.message}`));
    });
  });
}

function summary(samples) {
  const average = field => samples.reduce((sum, item) => sum + item[field], 0) / samples.length;
  return {
    rounds: samples.length,
    average_success_rps: Number(average('success_rps').toFixed(1)),
    average_latency_ms: Number(average('avg_ms').toFixed(3)),
    failed: samples.reduce((sum, item) => sum + item.failed, 0),
    samples,
  };
}

async function main() {
  for (const [name, path] of Object.entries(routes)) {
    await preflight(name, path);
  }

  // Warm both handlers before measurement so compilation, connection setup,
  // and tetherscript's per-blocking-thread plugin caches are not timed.
  await run(routes.rust, 250);
  await run(routes.tetherscript, 250);

  const samples = { rust: [], tetherscript: [] };
  for (let round = 0; round < rounds; round++) {
    const order = round % 2 ? ['tetherscript', 'rust'] : ['rust', 'tetherscript'];
    for (const name of order) samples[name].push(await run(routes[name], requests));
  }
  console.log(JSON.stringify({
    workload: { requests_per_round: requests, concurrency, rounds },
    rust: summary(samples.rust),
    tetherscript: summary(samples.tetherscript),
  }, null, 2));
}

main().catch(error => { console.error(error); process.exitCode = 1; });