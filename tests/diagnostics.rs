use std::fs;

use chrono::NaiveDate;
use tempfile::tempdir;
use wt_presence::diagnostics::prune_old_logs;

#[test]
fn log_retention_removes_only_expired_wt_presence_logs() {
    let directory = tempdir().unwrap();
    let expired = directory.path().join("wt-presence.log.2026-09-29");
    let retained = directory.path().join("wt-presence.log.2026-09-30");
    let unrelated = directory.path().join("notes.txt");
    fs::write(&expired, "old").unwrap();
    fs::write(&retained, "current").unwrap();
    fs::write(&unrelated, "leave me alone").unwrap();

    let removed = prune_old_logs(
        directory.path(),
        NaiveDate::from_ymd_opt(2026, 10, 6).unwrap(),
        7,
    )
    .unwrap();

    assert_eq!(removed, 1);
    assert!(!expired.exists());
    assert!(retained.exists());
    assert!(unrelated.exists());
}
