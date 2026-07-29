//! Built-in files extracted to `~/.weepcode/` on startup.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const BUNDLED_FILES: &[(&str, &str)] = &[("README.md", include_str!("../README.md"))];

const BUNDLED_SKILL_MANIFEST_FILENAME: &str = ".bundled_skill_manifest.json";

/// Hashes produced by the first merged Grok-skill release before WeepCode's
/// runtime-contract fixes. They let the next same-version startup recognize
/// those files as managed and upgrade them without claiming user-authored
/// files that happen to use the same skill name.
const PRE_FIX_BUNDLED_SKILL_HASHES: &[(&str, &str)] = &[
    (
        "skills/review/SKILL.md",
        "1e93972125ef8b8bab9f963dfa23eaea4c26e854cc196d562d1cf3b23ada8594",
    ),
    (
        "skills/design/SKILL.md",
        "9fc5bb7e3a4e96f3bb094505d9f9ee55308114c170edc536788dd2511fc43f19",
    ),
];

#[derive(Debug, Default, Deserialize, Serialize)]
struct BundledSkillManifest {
    /// Relative path under `weepcode_home` -> SHA-256 of the last content
    /// successfully written by WeepCode.
    files: BTreeMap<String, String>,
}

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

/// True when a discovered skill is an unmodified copy managed by the bundled
/// skill manifest. A user-authored skill at the same path is preserved and is
/// therefore not mislabeled as bundled by `weepcode inspect`.
pub(crate) fn is_extracted_bundled_skill(
    name: &str,
    path: &std::path::Path,
    weepcode_home: &std::path::Path,
) -> bool {
    if !BUNDLED_SKILLS
        .iter()
        .any(|&(bundled_name, _)| bundled_name == name)
        || path != weepcode_home.join("skills").join(name).join("SKILL.md")
    {
        return false;
    }

    let relative_path = format!("skills/{name}/SKILL.md");
    let manifest = load_bundled_skill_manifest(weepcode_home);
    let Some(managed_hash) = manifest.files.get(&relative_path) else {
        return false;
    };
    std::fs::read(path)
        .map(|content| bundled_content_hash(&content) == *managed_hash)
        .unwrap_or(false)
}

/// Resolve the content for a skill, applying WeepCode-specific adaptations.
fn resolve_skill_content(name: &str, raw: &str, weepcode_home: &std::path::Path) -> String {
    match name {
        "help" => {
            let weepcode_home_str = format!("{}/", weepcode_home.to_string_lossy());
            raw.replace("~/.weepcode/", &weepcode_home_str)
        }
        "create-workflow" => raw
            .replace("Grok Build", "WeepCode")
            .replace(".grok/", ".weepcode/"),
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
/// Skill files are reconciled through a hash manifest on every startup. Files
/// still matching the last WeepCode-written hash can be upgraded or repaired;
/// files modified or created by the user are never overwritten.
pub fn extract_bundled_files(weepcode_home: &std::path::Path) {
    remove_legacy_bundled_skills(weepcode_home);

    let version = weepcode_version::VERSION;
    let version_marker = weepcode_home.join(".metadata_version");
    let is_same_version = std::fs::read_to_string(&version_marker)
        .map(|existing| existing.trim() == version)
        .unwrap_or(false);

    let _ = std::fs::create_dir_all(weepcode_home);
    let mut skill_manifest = load_bundled_skill_manifest(weepcode_home);

    if !is_same_version {
        for stale in &["CHANGELOG.json", "CHANGELOG.md"] {
            let _ = std::fs::remove_file(weepcode_home.join(stale));
        }

        for &(filename, content) in BUNDLED_FILES {
            if let Err(error) = std::fs::write(weepcode_home.join(filename), content) {
                tracing::debug!(error = %error, filename, "Failed to extract bundled file");
            }
        }
    }

    reconcile_bundled_skills(weepcode_home, &mut skill_manifest);
    save_bundled_skill_manifest(weepcode_home, &skill_manifest);

    if !is_same_version {
        let _ = std::fs::write(&version_marker, version);
        tracing::debug!(version, "Extracted bundled files");
    }
}

fn reconcile_bundled_skills(
    weepcode_home: &std::path::Path,
    skill_manifest: &mut BundledSkillManifest,
) {
    let mut managed_skill_names = BTreeMap::new();
    for &(name, raw_content) in BUNDLED_SKILLS {
        let relative_path = format!("skills/{name}/SKILL.md");
        let resolved_content = resolve_skill_content(name, raw_content, weepcode_home);
        let is_managed = reconcile_bundled_skill_file(
            weepcode_home,
            &relative_path,
            resolved_content.as_bytes(),
            skill_manifest,
        );
        managed_skill_names.insert(name, is_managed);
    }

    for &(support_relative_path, content) in BUNDLED_SKILL_SUPPORT_FILES {
        let owner_name = bundled_support_file_owner(support_relative_path);
        if !managed_skill_names
            .get(owner_name)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let relative_path = format!("skills/{support_relative_path}");
        reconcile_bundled_skill_file(weepcode_home, &relative_path, content, skill_manifest);
    }
}

fn bundled_support_file_owner(relative_path: &str) -> &str {
    if relative_path.starts_with("pdf/") {
        "pdf"
    } else if relative_path == "shared/personas/reviewer.toml" {
        "review"
    } else {
        "design"
    }
}

fn reconcile_bundled_skill_file(
    weepcode_home: &std::path::Path,
    relative_path: &str,
    bundled_content: &[u8],
    skill_manifest: &mut BundledSkillManifest,
) -> bool {
    let destination = weepcode_home.join(relative_path);
    let bundled_hash = bundled_content_hash(bundled_content);
    let existing_content = std::fs::read(&destination).ok();
    let existing_hash = existing_content.as_deref().map(bundled_content_hash);
    let recorded_hash = skill_manifest.files.get(relative_path);
    let pre_fix_hash = PRE_FIX_BUNDLED_SKILL_HASHES
        .iter()
        .find_map(|(path, hash)| (*path == relative_path).then_some(*hash));

    let is_safe_to_manage = match existing_hash.as_deref() {
        None => true,
        Some(hash) if hash == bundled_hash => true,
        Some(hash) if recorded_hash.is_some_and(|recorded| recorded == hash) => true,
        Some(hash) if pre_fix_hash.is_some_and(|previous| previous == hash) => true,
        Some(_) => false,
    };

    if !is_safe_to_manage {
        tracing::warn!(
            path = relative_path,
            "Preserving user-modified file that conflicts with a bundled skill"
        );
        return false;
    }

    if existing_hash.as_deref() != Some(bundled_hash.as_str()) {
        if let Some(parent) = destination.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            tracing::debug!(error = %error, path = relative_path, "Failed to create bundled skill directory");
            return false;
        }
        if let Err(error) = std::fs::write(&destination, bundled_content) {
            tracing::debug!(error = %error, path = relative_path, "Failed to write bundled skill file");
            return false;
        }
    }

    skill_manifest
        .files
        .insert(relative_path.to_string(), bundled_hash);
    true
}

fn bundled_content_hash(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn load_bundled_skill_manifest(weepcode_home: &std::path::Path) -> BundledSkillManifest {
    let path = weepcode_home.join(BUNDLED_SKILL_MANIFEST_FILENAME);
    std::fs::read(&path)
        .ok()
        .and_then(|content| serde_json::from_slice(&content).ok())
        .unwrap_or_default()
}

fn save_bundled_skill_manifest(
    weepcode_home: &std::path::Path,
    skill_manifest: &BundledSkillManifest,
) {
    let path = weepcode_home.join(BUNDLED_SKILL_MANIFEST_FILENAME);
    let Ok(content) = serde_json::to_vec_pretty(skill_manifest) else {
        return;
    };
    if let Err(error) = std::fs::write(&path, content) {
        tracing::debug!(error = %error, "Failed to persist bundled skill manifest");
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
    fn version_bump_preserves_user_modified_skill_files() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);

        let review_path = home.join("skills/review/SKILL.md");
        let support_path = home.join("skills/pdf/reference.md");
        std::fs::write(&review_path, "user review skill").unwrap();
        std::fs::write(&support_path, "user pdf reference").unwrap();
        std::fs::write(home.join(".metadata_version"), "0.0.0-stale").unwrap();

        extract_bundled_files(home);

        assert_eq!(
            std::fs::read_to_string(&review_path).unwrap(),
            "user review skill"
        );
        assert_eq!(
            std::fs::read_to_string(&support_path).unwrap(),
            "user pdf reference"
        );
        assert!(!is_extracted_bundled_skill("review", &review_path, home));
    }

    #[test]
    fn version_bump_refreshes_unmodified_managed_files_and_removes_legacy_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();

        extract_bundled_files(home);
        let review_path = home.join("skills/review/SKILL.md");
        assert!(is_extracted_bundled_skill("review", &review_path, home));

        for name in ["check", "best-of-n", "docx", "pptx", "xlsx"] {
            std::fs::create_dir_all(home.join(format!("skills/{name}"))).unwrap();
            std::fs::write(
                home.join(format!("skills/{name}/SKILL.md")),
                "old legacy skill",
            )
            .unwrap();
        }
        std::fs::write(home.join(".metadata_version"), "0.0.0-stale").unwrap();

        extract_bundled_files(home);

        assert!(is_extracted_bundled_skill("review", &review_path, home));
        for name in ["check", "best-of-n", "docx", "pptx", "xlsx"] {
            assert!(
                !home.join(format!("skills/{name}")).exists(),
                "legacy '{name}' skill directory should have been deleted"
            );
        }
    }

    #[test]
    fn existing_custom_skill_does_not_receive_bundled_support_files() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let custom_pdf_dir = home.join("skills/pdf");
        std::fs::create_dir_all(&custom_pdf_dir).unwrap();
        std::fs::write(custom_pdf_dir.join("SKILL.md"), "custom pdf skill").unwrap();

        extract_bundled_files(home);

        assert_eq!(
            std::fs::read_to_string(custom_pdf_dir.join("SKILL.md")).unwrap(),
            "custom pdf skill"
        );
        assert!(!custom_pdf_dir.join("reference.md").exists());
        assert!(!custom_pdf_dir.join("forms/f1040--2025.pdf").exists());
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
        assert!(review_content.contains("run_terminal_command"));
        assert!(!review_content.contains("run_terminal_cmd`"));
        assert!(review_content.contains("`background`: `false`"));
        assert!(review_content.contains("`capability_mode`: `\"read-only\"`"));
        assert!(review_content.contains("git worktree add --detach"));

        let design_content = std::fs::read_to_string(home.join("skills/design/SKILL.md")).unwrap();
        assert!(design_content.contains("../shared/personas/design-doc-writer.toml"));
        assert!(design_content.contains("../shared/personas/design-doc-reviewer.toml"));
        assert!(!design_content.contains("../shared/personas/design-doc-writer.md"));
        assert!(!design_content.contains("../shared/personas/design-doc-reviewer.md"));
        assert!(design_content.contains("run_terminal_command"));
        assert!(!design_content.contains("run_terminal_cmd`"));
        assert!(design_content.contains("`background`: `false`"));
        assert!(design_content.contains("`capability_mode`: `\"read-only\"`"));
        assert!(design_content.contains("Cap automatic review/revision"));

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
