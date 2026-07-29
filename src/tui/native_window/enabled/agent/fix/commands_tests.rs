//! Fix Runner command evidence tests.

#[test]
fn validation_captures_success_and_failure_evidence() {
    let success = super::shell("echo fix-runner-validation").unwrap();
    assert!(success.contains("exit: exit code: 0"));
    assert!(success.contains("fix-runner-validation"));
    assert!(super::shell("exit 7").unwrap_err().contains("exit code: 7"));
}

#[test]
fn git_status_captures_exit_evidence() {
    let evidence = super::run("git", &["status", "--short"]).unwrap();
    assert!(evidence.starts_with("exit: exit code: 0"));
}

#[test]
fn asynchronous_jobs_deliver_git_and_validation_output() {
    let context = eframe::egui::Context::default();
    let mut validation = super::super::job::Job::idle("validation");
    super::validate(&mut validation, "echo async-validation", &context);
    wait(&mut validation);
    assert!(validation.output.contains("async-validation"));
    let mut git = super::super::job::Job::idle("git");
    super::refresh(&mut git, &context);
    wait(&mut git);
    assert!(git.output.contains("GIT STATUS"));
    assert!(git.output.contains("UNIFIED DIFF"));
}

fn wait(job: &mut super::super::job::Job) {
    for _ in 0..100 {
        job.poll();
        if !job.running {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("background job did not finish");
}
