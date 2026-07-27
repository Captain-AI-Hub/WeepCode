//! `WEEPCODE_HOME` override tests in an isolated binary so `weepcode_home()`'s
//! process-wide `OnceLock` initializes from the overridden env var.

use std::path::PathBuf;

#[test]
fn weepcode_home_override_path_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let weepcode_home = tmp.path().to_path_buf();
    unsafe {
        std::env::set_var("WEEPCODE_HOME", &weepcode_home);
    }

    assert_eq!(
        weepcode_pager::util::pager_toml_path(),
        weepcode_home.join("pager.toml")
    );
    assert_eq!(
        weepcode_pager::util::display_weepcode_home_prefix(),
        "$WEEPCODE_HOME"
    );
    assert_eq!(
        weepcode_pager::util::display_user_weepcode_path("config.toml"),
        "$WEEPCODE_HOME/config.toml"
    );

    let memory_path = weepcode_home.join("memory/MEMORY.md");
    assert_eq!(
        weepcode_pager::util::abbreviate_path(&memory_path.display().to_string()),
        "$WEEPCODE_HOME/memory/MEMORY.md"
    );

    assert!(weepcode_pager::util::is_under_user_weepcode_home(
        &memory_path
    ));
    assert!(!weepcode_pager::util::is_under_user_weepcode_home(
        PathBuf::from("/tmp/other").as_path()
    ));
}
