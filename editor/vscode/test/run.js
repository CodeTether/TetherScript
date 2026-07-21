const tests = [
  ['module aliases', require('./module-alias.test')],
  ['module syntax', require('./module-syntax.test')],
  ['module paths', require('./module-path.test')],
  ['source masking', require('./source-mask.test')],
];

async function main() {
  let failed = false;
  for (const [name, test] of tests) {
    try {
      await test();
      console.log(`ok - ${name}`);
    } catch (error) {
      failed = true;
      console.error(`not ok - ${name}\n${error.stack}`);
    }
  }
  if (failed) process.exitCode = 1;
}

void main();