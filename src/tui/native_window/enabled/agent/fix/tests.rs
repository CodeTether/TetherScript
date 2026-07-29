//! Fix Runner task-contract regression tests.

#[test]
fn task_prompt_requires_work_evidence_without_delivery_side_effects() {
    let prompt = super::prompt::build("repair the parser");
    assert!(prompt.contains("repair the parser"));
    assert!(prompt.contains("Do actual work"));
    assert!(prompt.contains("Do not commit or push"));
    assert!(prompt.contains("commands run"));
}
