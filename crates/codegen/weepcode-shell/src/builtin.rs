//! Built-in files extracted to `~/.weepcode/` on startup.

const BUNDLED_FILES: &[(&str, &str)] = &[("README.md", include_str!("../README.md"))];

const HELP_SKILL_MD: &str = include_str!("../skills/help/SKILL.md");
const CREATE_SKILL_MD: &str = include_str!("../skills/create-skill/SKILL.md");
const CREATE_WORKFLOW_SKILL_MD: &str = include_str!("../skills/create-workflow/SKILL.md");
const CODE_REVIEW_SKILL_MD: &str = include_str!("../skills/code-review/SKILL.md");
const REVIEW_SKILL_MD: &str = include_str!("../skills/review/SKILL.md");
const DESIGN_SKILL_MD: &str = include_str!("../skills/design/SKILL.md");
const PDF_SKILL_MD: &str = include_str!("../skills/pdf/SKILL.md");
const IMAGINE_SKILL_MD: &str = include_str!("../skills/imagine/SKILL.md");
/// Compiled-in SKILL.md content for `/check-work` (available to headless mode).
pub const CHECK_SKILL_MD: &str = include_str!("../skills/check-work/SKILL.md");
/// Compiled-in SKILL.md content for headless `--best-of-n` (not extracted as
/// a bundled skill).
pub const BEST_OF_N_SKILL_MD: &str = include_str!("../skills/best-of-n/SKILL.md");

macro_rules! bundled_skill_support_file {
    ($relative_path:literal) => {
        (
            $relative_path,
            include_bytes!(concat!("../skills/", $relative_path)) as &[u8],
        )
    };
}

/// Legacy bundled skill names (renamed or removed).
///
/// These directories under `~/.weepcode/skills/` will be deleted on startup
/// (during bundled file extraction). This ensures that when a bundled
/// skill is renamed (e.g. `check` → `check-work`), the old slash command
/// does not linger on users' machines after an upgrade.
///
/// Important behavior:
/// - Deletion happens **early** in `extract_bundled_files`, before we write
///   any current bundled skills.
/// - We **never** delete a name that is currently present in `BUNDLED_SKILLS`
///   (see `remove_legacy_bundled_skills`).
///
/// This means:
/// - If you later re-introduce a skill with a name that is still in this
///   legacy list (e.g. you ship a new "check" skill years later), the legacy
///   cleanup will **skip** it and the new skill will be created normally.
/// - The legacy list is a "delete old user copies of names we no longer ship",
///   not a permanent blacklist.
///
/// Lifecycle / maintenance:
/// - Add an old name here when you rename/remove a bundled skill.
/// - Once the directory is gone on a user's machine, further checks are
///   cheap no-ops.
/// - You do **not** have to remove entries immediately. It is safe to leave
///   them for many releases.
/// - After the rename has had time to propagate, you **may** clean old
///   strings out of this list for hygiene.
const LEGACY_BUNDLED_SKILL_NAMES: &[&str] = &["check", "best-of-n", "docx", "pptx", "xlsx"];

/// All bundled skill SKILL.md files. Single source of truth used by both
/// the full extraction path (version bump) and the missing-file fast path
/// (same version). Adding a new skill here is all that's needed.
///
/// When renaming a bundled skill (e.g. "check" → "check-work"), also add the
/// old name to `LEGACY_BUNDLED_SKILL_NAMES` so `remove_legacy_bundled_skills`
/// will clean up the old directory on user machines on the next upgrade.
///
/// See the docs on `LEGACY_BUNDLED_SKILL_NAMES` for the full lifecycle
/// (including when it is safe/optional to remove old entries later).
const BUNDLED_SKILLS: &[(&str, &str)] = &[
    ("help", HELP_SKILL_MD),
    ("create-skill", CREATE_SKILL_MD),
    ("create-workflow", CREATE_WORKFLOW_SKILL_MD),
    ("code-review", CODE_REVIEW_SKILL_MD),
    ("review", REVIEW_SKILL_MD),
    ("design", DESIGN_SKILL_MD),
    ("pdf", PDF_SKILL_MD),
    ("imagine", IMAGINE_SKILL_MD),
    ("check-work", CHECK_SKILL_MD),
];

/// Nested files required by bundled skills.
///
/// Paths are relative to `<weepcode_home>/skills/`. Unlike `SKILL.md`, these
/// resources can be binary (the PDF skill ships official form templates), so
/// they are embedded with `include_bytes!`.
const BUNDLED_SKILL_SUPPORT_FILES: &[(&str, &[u8])] = &[
    bundled_skill_support_file!("shared/personas/reviewer.toml"),
    bundled_skill_support_file!("shared/personas/design-doc-writer.toml"),
    bundled_skill_support_file!("shared/personas/design-doc-reviewer.toml"),
    bundled_skill_support_file!("pdf/forms.md"),
    bundled_skill_support_file!("pdf/reference.md"),
    bundled_skill_support_file!("pdf/tax.md"),
    bundled_skill_support_file!("pdf/scripts/check_bounding_boxes.py"),
    bundled_skill_support_file!("pdf/scripts/check_fillable_fields.py"),
    bundled_skill_support_file!("pdf/scripts/convert_pdf_to_images.py"),
    bundled_skill_support_file!("pdf/scripts/create_validation_image.py"),
    bundled_skill_support_file!("pdf/scripts/extract_form_field_info.py"),
    bundled_skill_support_file!("pdf/scripts/extract_form_structure.py"),
    bundled_skill_support_file!("pdf/scripts/fill_fillable_fields.py"),
    bundled_skill_support_file!("pdf/scripts/fill_pdf_form_with_annotations.py"),
    bundled_skill_support_file!("pdf/forms/f1040--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040s1--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040s2--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040s3--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040s8--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040sa--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040sb--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040sc--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040sd--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040se--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f1040sse--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f2441--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f5329--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8812--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8889--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8936--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8949--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8959--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8960--2025.pdf"),
    bundled_skill_support_file!("pdf/forms/f8995--2025.pdf"),
];

/// True when a discovered skill is the copy `extract_bundled_files` wrote to
/// `<weepcode_home>/skills/<name>/SKILL.md`. Exact-path (not prefix) so a
/// user-authored skill that reuses a bundled name — even elsewhere under
/// `<weepcode_home>/skills/` — is never labeled bundled. Lives beside the
/// extraction code so the target layout and this predicate move together.
/// Used by inspect, which otherwise sees extracted copies as user skills.
pub(crate) fn is_extracted_bundled_skill(
    name: &str,
    path: &std::path::Path,
    weepcode_home: &std::path::Path,
) -> bool {
    BUNDLED_SKILLS.iter().any(|&(n, _)| n == name)
        && path == weepcode_home.join("skills").join(name).join("SKILL.md")
}

/// Resolve the content for a skill, applying any name-specific transforms.
fn resolve_skill_content(name: &str, raw: &str, weepcode_home: &std::path::Path) -> String {
    match name {
        // Help skill needs path substitution so absolute paths work.
        "help" => {
            let weepcode_home_str = format!("{}/", weepcode_home.to_string_lossy());
            raw.replace("~/.weepcode/", &weepcode_home_str)
        }
        // Keep the bundled source byte-for-byte aligned with the Grok release
        // artifact; only adapt product names and workflow paths on extraction.
        "create-workflow" => raw
            .replace("Grok Build", "WeepCode")
            .replace(".grok/", ".weepcode/"),
        // The Grok archive stores personas as TOML in its bundled persona
        // catalog, while these two skills still reference the older Markdown
        // support-file layout. Keep the vendored skill source unchanged and
        // point the extracted copy at the byte-identical TOML resources.
        "review" => raw.replace(
            "../shared/personas/reviewer.md",
            "../shared/personas/reviewer.toml",
        ),
        "design" => raw
            .replace(
                "../shared/personas/design-doc-writer.md",
                "../shared/personas/design-doc-writer.toml",
            )
            .replace(
                "../shared/personas/design-doc-reviewer.md",
                "../shared/personas/design-doc-reviewer.toml",
            ),
        _ => raw.to_string(),
    }
}

/// Extract bundled files to `~/.weepcode/` on startup.
///
/// Full extraction runs on every version bump. On same-version startups,
/// a lightweight check ensures all expected skill files exist on disk —
/// any missing files are extracted individually.
///
/// Legacy/renamed bundled skills (see `LEGACY_BUNDLED_SKILL_NAMES`) are
/// always cleaned up first so that old slash commands disappear after
/// a rename (e.g. the previous `/check` after the move to `/check-work`).
pub fn extract_bundled_files(weepcode_home: &std::path::Path) {
    // Always remove legacy/renamed bundled skills first (e.g. the old
    // `check` directory after the rename to `check-work`). This runs on
    // every startup so users get cleaned up even without hitting a
    // version-bump marker change.
    remove_legacy_bundled_skills(weepcode_home);

    let version = weepcode_version::VERSION;
    let marker = weepcode_home.join(".metadata_version");

    if let Ok(existing) = std::fs::read_to_string(&marker)
        && existing.trim() == version
    {
        // Same version — only extract skill files that are missing on disk.
        // This handles skills added between version bumps.
        extract_missing_skills(weepcode_home);
        return;
    }

    let _ = std::fs::create_dir_all(weepcode_home);

    // Clean up cached changelog files from previous version so
    // /release-notes fetches fresh content for the new version.
    for stale in &["CHANGELOG.json", "CHANGELOG.md"] {
        let _ = std::fs::remove_file(weepcode_home.join(stale));
    }

    for &(filename, content) in BUNDLED_FILES {
        if let Err(e) = std::fs::write(weepcode_home.join(filename), content) {
            tracing::debug!(error = %e, filename, "Failed to extract bundled file");
        }
    }

    // Skill SKILL.md files.
    for &(name, raw) in BUNDLED_SKILLS {
        let skill_dir = weepcode_home.join("skills").join(name);
        let _ = std::fs::create_dir_all(&skill_dir);
        let content = resolve_skill_content(name, raw, weepcode_home);
        if let Err(e) = std::fs::write(skill_dir.join("SKILL.md"), content) {
            tracing::debug!(error = %e, name, "Failed to write skill");
        }
    }
    extract_skill_support_files(weepcode_home, false);

    let _ = std::fs::write(&marker, version);
    tracing::debug!(version, "Extracted bundled files");
}

/// Extract only missing skill SKILL.md files (same-version fast path).
/// Iterates `BUNDLED_SKILLS` so adding a new skill there is sufficient.
fn extract_missing_skills(weepcode_home: &std::path::Path) {
    for &(name, raw) in BUNDLED_SKILLS {
        let skill_md = weepcode_home.join("skills").join(name).join("SKILL.md");
        if skill_md.exists() {
            continue;
        }
        let _ = std::fs::create_dir_all(skill_md.parent().unwrap());
        let content = resolve_skill_content(name, raw, weepcode_home);
        let _ = std::fs::write(&skill_md, content);
    }
    extract_skill_support_files(weepcode_home, true);
}

/// Extract nested skill resources.
///
/// On a version bump all managed resources are refreshed. The same-version
/// fast path writes only missing files, matching the repair behavior for
/// top-level `SKILL.md` files.
fn extract_skill_support_files(weepcode_home: &std::path::Path, only_missing: bool) {
    let skills_dir = weepcode_home.join("skills");
    for &(relative_path, content) in BUNDLED_SKILL_SUPPORT_FILES {
        let destination = skills_dir.join(relative_path);
        if only_missing && destination.exists() {
            continue;
        }
        if let Some(parent) = destination.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(error) = std::fs::write(&destination, content) {
            tracing::debug!(
                error = %error,
                path = %relative_path,
                "Failed to write bundled skill support file"
            );
        }
    }
}

/// Remove directories for legacy/renamed bundled skills (e.g. old `check`
/// after it was renamed to `check-work`).
///
/// Called on every startup from `extract_bundled_files`. Safe and idempotent.
///
/// Key guarantees (see `LEGACY_BUNDLED_SKILL_NAMES` docs for details):
/// - If a name is still present in `BUNDLED_SKILLS`, we deliberately skip
///   deletion. This allows safe re-use of a skill name in the future.
/// - If the target directory no longer exists, this is a trivial no-op.
fn remove_legacy_bundled_skills(weepcode_home: &std::path::Path) {
    remove_legacy_skills(weepcode_home, LEGACY_BUNDLED_SKILL_NAMES, BUNDLED_SKILLS);
}

/// Core implementation, extracted for testability.
fn remove_legacy_skills(
    weepcode_home: &std::path::Path,
    legacy_names: &[&str],
    bundled_skills: &[(&str, &str)],
) {
    for name in legacy_names {
        // Safety: Never delete a name that we are currently shipping.
        // This protects against re-introducing a skill name that still has
        // an entry in the legacy list.
        if bundled_skills.iter().any(|(n, _)| *n == *name) {
            continue;
        }

        let dir = weepcode_home.join("skills").join(name);
        if dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                tracing::debug!(error = %e, name, "Failed to remove legacy bundled skill");
            } else {
                tracing::debug!(name, "Removed legacy bundled skill directory");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_bump_re_extracts_all_files() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        for &(filename, _) in BUNDLED_FILES {
            std::fs::write(home.join(filename), "old").unwrap();
        }
        for &(name, _) in BUNDLED_SKILLS {
            std::fs::write(home.join(format!("skills/{name}/SKILL.md")), "old").unwrap();
        }
        for &(relative_path, _) in BUNDLED_SKILL_SUPPORT_FILES {
            std::fs::write(home.join("skills").join(relative_path), "old").unwrap();
        }
        std::fs::write(home.join(".metadata_version"), "0.0.0-stale").unwrap();

        // Simulate legacy skills that should be cleaned up.
        for name in ["check", "best-of-n", "docx", "pptx", "xlsx"] {
            std::fs::create_dir_all(home.join(format!("skills/{name}"))).unwrap();
            std::fs::write(
                home.join(format!("skills/{name}/SKILL.md")),
                "old legacy skill",
            )
            .unwrap();
        }

        extract_bundled_files(home);

        for &(filename, _) in BUNDLED_FILES {
            assert_ne!(
                std::fs::read_to_string(home.join(filename)).unwrap(),
                "old",
                "{filename} was not re-extracted after version bump"
            );
        }
        for &(name, _) in BUNDLED_SKILLS {
            assert_ne!(
                std::fs::read_to_string(home.join(format!("skills/{name}/SKILL.md"))).unwrap(),
                "old",
                "{name} skill was not re-extracted after version bump"
            );
        }
        for &(relative_path, _) in BUNDLED_SKILL_SUPPORT_FILES {
            assert_ne!(
                std::fs::read(home.join("skills").join(relative_path)).unwrap(),
                b"old",
                "{relative_path} was not re-extracted after version bump"
            );
        }

        // Legacy skill directories must have been removed (the key part of
        // supporting renames like check → check-work without leaving orphans).
        for name in ["check", "best-of-n", "docx", "pptx", "xlsx"] {
            assert!(
                !home.join(format!("skills/{name}")).exists(),
                "legacy '{name}' skill directory should have been deleted during version bump"
            );
        }
    }

    #[test]
    fn office_skills_not_bundled() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        // Former office document skills must NOT be extracted as bundled.
        for name in ["docx", "pptx", "xlsx"] {
            assert!(
                !home.join(format!("skills/{name}")).exists(),
                "{name} should not be a bundled skill"
            );
        }
    }

    #[tokio::test]
    async fn help_skill_discovered_by_skill_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        let workspace = tmp.path().join("workspace");
        std::fs::create_dir_all(workspace.join(".weepcode").join("skills").join("help")).unwrap();
        std::fs::copy(
            home.join("skills/help/SKILL.md"),
            workspace.join(".weepcode/skills/help/SKILL.md"),
        )
        .unwrap();

        let skills = weepcode_agent::prompt::skills::list_skills(
            Some(workspace.to_str().unwrap()),
            &Default::default(),
            weepcode_agent::prompt::skills::CompatConfig::default(),
        )
        .await;

        let help = skills.iter().find(|s| s.name == "help");
        assert!(
            help.is_some(),
            "help skill not found. skills: {:?}",
            skills.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        let help = help.unwrap();
        assert!(help.description.contains("configuration"));
        assert!(help.user_invocable);
    }

    #[tokio::test]
    async fn create_workflow_skill_is_extracted_and_discovered() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        let extracted_skill = home.join("skills/create-workflow/SKILL.md");
        let extracted_content = std::fs::read_to_string(&extracted_skill).unwrap();
        assert!(extracted_content.contains("Create a WeepCode workflow"));
        assert!(extracted_content.contains("~/.weepcode/workflows/"));
        assert!(!extracted_content.contains("Grok Build"));
        assert!(!extracted_content.contains(".grok/"));

        let workspace = tmp.path().join("workspace");
        let project_skill_dir = workspace.join(".weepcode/skills/create-workflow");
        std::fs::create_dir_all(&project_skill_dir).unwrap();
        std::fs::copy(extracted_skill, project_skill_dir.join("SKILL.md")).unwrap();

        let skills = weepcode_agent::prompt::skills::list_skills(
            Some(workspace.to_str().unwrap()),
            &Default::default(),
            weepcode_agent::prompt::skills::CompatConfig::default(),
        )
        .await;

        let create_workflow = skills.iter().find(|skill| skill.name == "create-workflow");
        assert!(
            create_workflow.is_some(),
            "create-workflow skill not found. skills: {:?}",
            skills.iter().map(|skill| &skill.name).collect::<Vec<_>>()
        );
        let create_workflow = create_workflow.unwrap();
        assert!(
            create_workflow
                .description
                .contains("Rhai orchestration script")
        );
        assert!(create_workflow.user_invocable);
    }

    #[tokio::test]
    async fn review_design_and_pdf_skills_are_extracted_with_support_files() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        let review_content = std::fs::read_to_string(home.join("skills/review/SKILL.md")).unwrap();
        assert!(review_content.contains("../shared/personas/reviewer.toml"));
        assert!(!review_content.contains("../shared/personas/reviewer.md"));

        let design_content = std::fs::read_to_string(home.join("skills/design/SKILL.md")).unwrap();
        assert!(design_content.contains("../shared/personas/design-doc-writer.toml"));
        assert!(design_content.contains("../shared/personas/design-doc-reviewer.toml"));
        assert!(!design_content.contains("../shared/personas/design-doc-writer.md"));
        assert!(!design_content.contains("../shared/personas/design-doc-reviewer.md"));

        assert!(home.join("skills/shared/personas/reviewer.toml").is_file());
        assert!(
            home.join("skills/shared/personas/design-doc-writer.toml")
                .is_file()
        );
        assert!(
            home.join("skills/shared/personas/design-doc-reviewer.toml")
                .is_file()
        );
        assert!(home.join("skills/pdf/reference.md").is_file());
        assert!(
            home.join("skills/pdf/scripts/convert_pdf_to_images.py")
                .is_file()
        );
        assert!(
            std::fs::read(home.join("skills/pdf/forms/f1040--2025.pdf"))
                .unwrap()
                .starts_with(b"%PDF")
        );

        let workspace = tmp.path().join("workspace");
        for skill_name in ["review", "design", "pdf"] {
            let project_skill_dir = workspace.join(".weepcode/skills").join(skill_name);
            std::fs::create_dir_all(&project_skill_dir).unwrap();
            std::fs::copy(
                home.join("skills").join(skill_name).join("SKILL.md"),
                project_skill_dir.join("SKILL.md"),
            )
            .unwrap();
        }

        let skills = weepcode_agent::prompt::skills::list_skills(
            Some(workspace.to_str().unwrap()),
            &Default::default(),
            weepcode_agent::prompt::skills::CompatConfig::default(),
        )
        .await;

        for skill_name in ["review", "design", "pdf"] {
            let skill = skills.iter().find(|skill| skill.name == skill_name);
            assert!(
                skill.is_some(),
                "{skill_name} skill not found. skills: {:?}",
                skills.iter().map(|skill| &skill.name).collect::<Vec<_>>()
            );
            assert!(skill.unwrap().user_invocable);
        }
    }

    #[test]
    fn same_version_repairs_missing_skill_support_files() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        let missing_paths = [
            "skills/shared/personas/reviewer.toml",
            "skills/pdf/reference.md",
            "skills/pdf/forms/f1040--2025.pdf",
        ];
        for relative_path in missing_paths {
            std::fs::remove_file(home.join(relative_path)).unwrap();
        }
        std::fs::write(home.join(".metadata_version"), weepcode_version::VERSION).unwrap();

        extract_bundled_files(home);

        for relative_path in missing_paths {
            assert!(
                home.join(relative_path).is_file(),
                "{relative_path} should be repaired on same-version startup"
            );
        }
    }

    // ---------------------------------------------------------------------
    // Tests for legacy bundled skill removal (the rename migration system)
    // ---------------------------------------------------------------------

    #[test]
    fn remove_legacy_deletes_old_skill_when_not_currently_shipped() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        // Simulate an old legacy "check" directory from before a rename.
        let legacy_dir = home.join("skills/check");
        std::fs::create_dir_all(&legacy_dir).unwrap();
        std::fs::write(legacy_dir.join("SKILL.md"), "old check").unwrap();

        // "check" is in legacy list but NOT in current BUNDLED_SKILLS
        remove_legacy_skills(home, &["check"], BUNDLED_SKILLS);

        assert!(
            !legacy_dir.exists(),
            "legacy skill directory should have been deleted"
        );
    }

    #[test]
    fn remove_legacy_does_not_delete_when_name_is_reused_in_current_bundled() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        // User still has an old "check" directory.
        let legacy_dir = home.join("skills/check");
        std::fs::create_dir_all(&legacy_dir).unwrap();
        std::fs::write(legacy_dir.join("SKILL.md"), "user had old check").unwrap();

        // Simulate the situation where we later re-ship a skill named "check".
        // In this case the legacy entry should be ignored.
        let fake_bundled: &[(&str, &str)] = &[("check", "fake content"), ("help", "help")];

        remove_legacy_skills(home, &["check"], fake_bundled);

        // The directory must still exist (we did not nuke the user's copy
        // or a skill we're about to (re)create).
        assert!(
            legacy_dir.exists(),
            "should not delete a name that is currently being shipped"
        );
    }

    #[test]
    fn remove_legacy_handles_multiple_names_some_current_some_legacy() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        std::fs::create_dir_all(home.join("skills/old-renamed")).unwrap();
        std::fs::write(home.join("skills/old-renamed/SKILL.md"), "old").unwrap();

        std::fs::create_dir_all(home.join("skills/another-legacy")).unwrap();
        std::fs::write(home.join("skills/another-legacy/SKILL.md"), "old2").unwrap();

        // Current bundled skills include one name that used to be legacy
        let current: &[(&str, &str)] = &[("another-legacy", "now shipping again")];

        // Legacy list contains both the truly removed one and the reintroduced one
        remove_legacy_skills(home, &["old-renamed", "another-legacy"], current);

        assert!(
            !home.join("skills/old-renamed").exists(),
            "truly legacy name should be removed"
        );
        assert!(
            home.join("skills/another-legacy").exists(),
            "reintroduced name must not be deleted"
        );
    }

    #[test]
    fn remove_legacy_is_noop_when_directory_does_not_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        // No directory exists for the legacy name
        remove_legacy_skills(home, &["check"], BUNDLED_SKILLS);

        // Should not panic or create anything
        assert!(!home.join("skills/check").exists());
    }

    #[test]
    fn legacy_cleanup_runs_even_on_same_version_fast_path() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        // First run: extract current state
        extract_bundled_files(home);

        // Simulate user still having an old legacy directory
        let legacy_dir = home.join("skills/check");
        std::fs::create_dir_all(&legacy_dir).unwrap();
        std::fs::write(legacy_dir.join("SKILL.md"), "stale").unwrap();

        // Force the "same version" fast path by writing the current version marker
        let version = weepcode_version::VERSION;
        std::fs::write(home.join(".metadata_version"), version).unwrap();

        // This should still run legacy cleanup even though we're in fast path
        extract_bundled_files(home);

        assert!(
            !legacy_dir.exists(),
            "legacy cleanup must run even on same-version fast path"
        );
    }
}
