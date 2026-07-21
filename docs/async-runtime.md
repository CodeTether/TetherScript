# Cooperative async runtime

tetherscript schedules `async fn` calls as owned `task` resources. Calling an
async function does not execute its body. `await`, `join`, or `select` is the
cooperative scheduling point that drives work, and task results transfer to the
caller exactly once.

```tether
async fn fetch_one(id) {
    return id
}

fn main() {
    let first = spawn fetch_one(1)
    let second = spawn fetch_one(2)
    let values = join(first, second)
    println(values)
}
```

`spawn value` always returns a task. An existing async task is scheduled as-is;
an immediate value becomes an already-completed task. `join(a, b, ...)` awaits
every task in argument order and returns their values as a list.

`select([tasks])` returns `{ index, value }`. It selects the first already-ready
task, falling back to the first scheduled task when none is ready. Other tasks
remain untouched. Moving task handles into the temporary list makes ownership
explicit: `select([move pending, move ready])`.

Tasks share normal resource controls. `cancel()` prevents future execution,
and `set_deadline(milliseconds)` makes an expired `await` fail with a
task-qualified deadline error. `id()`, `state()`, and `is_complete()` expose
stable task state for supervisors.

The scheduler is dependency-free and deterministic. It is cooperative rather
than preemptive: user code runs until it returns or reaches another async
operation. Host resources such as child-process streams remain nonblocking and
report bounded `backpressure` instead of blocking the scheduler thread.
