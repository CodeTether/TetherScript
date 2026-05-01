# Source-emission experiment

Answers the question: **do models emit valid Kiln source that accomplishes
real agent tasks?** This is the prerequisite experiment for any decision
about whether to invest in streaming bytecode, custom tokenizers, or
distilled specialist models for Kiln generation.

## What it measures

Every trajectory lands in one of four categories, defined before the run:

| Category   | Meaning                                                    | Implication if dominant |
| ---        | ---                                                        | --- |
| SUCCESS    | Parses, executes cleanly, success-check passes.            | The bet works. Scale up. |
| TASK       | Runs cleanly but doesn't accomplish the task.              | Model reasoning gap, not a language problem. Better prompt or better model. |
| SEMANTIC   | Parses but runtime fails (wrong method, unhandled `?`, capability violation, …). | Capability surface is rough, or model needs training data on Kiln semantics. |
| SYNTACTIC  | Lex or parse error.                                        | Prompt formatting issue, *or* source emission is a wrong bet and bytecode should be explored. |

## Running one trajectory

```bash
# 1. Get the prompt for a model
kiln --prompt fetch-and-save > prompt.txt

# 2. Feed it to whatever model you're testing. Capture its reply
#    (which should be Kiln source) to a file:
cat prompt.txt | your-model-cli > model_output.tether

# 3. Classify:
kiln --experiment fetch-and-save model_output.tether
# → JSON record on stderr, program stdout on stdout
# → exit 0 on SUCCESS, 1 otherwise
```

The JSON record on stderr is the trajectory payload — task id, category,
detail, program stdout, workspace path, full source. That is the shape
the distillation pipeline expects to ingest.

## Running a suite

```bash
for f in outputs/*.tether; do
    kiln --experiment fetch-and-save "$f" 2>> trajectories.jsonl 1>/dev/null
done

# Count by category:
jq -r '.category' trajectories.jsonl | sort | uniq -c
```

## Adding tasks

Edit `src/experiment.rs::tasks()`. Each task is a `TaskSpec`: id, human
description, the exact prompt text, capability grants (fs / http origins),
and a success-check closure.

A good task for this suite is one where:
- success is checkable programmatically (file exists, file contains X, HTTP call returned N)
- the capability set is small enough to enumerate in the prompt
- the failure-mode distribution is informative (i.e., not all models will trivially pass or trivially fail)

## What not to do with this harness

- Don't use it as a benchmark leaderboard. The categories are diagnostic,
  not competitive. A model that SYNTACTICs 90% might be strictly better
  for a specific pipeline than one that TASKs 50%.
- Don't tune the prompt to get SUCCESS rates up without understanding
  the failure category shift. A prompt change that moves failures from
  SYNTACTIC to TASK is real progress; one that moves them from SEMANTIC
  to TASK by hiding errors is not.
