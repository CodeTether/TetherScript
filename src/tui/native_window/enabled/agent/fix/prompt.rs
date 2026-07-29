//! Repository-task prompt contract for evidence-backed agent work.

pub(super) fn build(task: &str) -> String {
    format!(
        "Complete this repository task:

{task}

         Inspect the workspace before editing. Preserve unrelated existing changes.          Implement the smallest complete fix, then run the narrowest relevant checks.          Do not commit or push. Finish with files changed, commands run, observed results,          and any unresolved blocker. Do actual work rather than returning a plan."
    )
}
