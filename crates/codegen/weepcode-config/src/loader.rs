//! TOML loading, layered merging, and `$VAR` expansion.
//!
//! The merged result is the **default** config; requirements layers
//! sit on top via [`crate::validation`].

use std::path::Path;

use crate::paths::{system_config_dir, user_weepcode_home};
use crate::validation::{load_requirements, load_system_requirements};
use crate::version_overrides::{self, apply_version_overrides};

/// Load and parse a TOML file, expanding `$VAR` references. Empty table if absent.
pub fn load_toml_file(path: &Path) -> std::io::Result<toml::Value> {
    match std::fs::read_to_string(path) {
        Ok(s) => match toml::from_str::<toml::Value>(&s) {
            Ok(mut v) => {
                expand_env_vars_in_toml(&mut v);
                Ok(v)
            }
            Err(e) => {
                // Built from the span, never from Display — Display echoes the
                // offending source line, which may carry a secret. Safe to log and
                // to return to a client.
                let detail = toml_error_detail(&s, &e);
                tracing::error!(file = %path.display(), "config toml has syntax errors: {detail}");
                Err(std::io::Error::other(detail))
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(toml::Value::Table(toml::map::Map::new()))
        }
        Err(e) => {
            tracing::error!(file = %path.display(), "config file unreadable: {e}");
            Err(e)
        }
    }
}

/// A snippet-free description of a TOML parse error: `"TOML parse error at line
/// L, column C: <what>"` (or just the message when there's no span). Never
/// includes the offending source line — `Display` echoes it and it may carry a
/// secret — so this is safe to log or surface to a client. Shared with the trace
/// `config_files` artifact so the redaction rule lives in one place.
pub fn toml_error_detail(src: &str, e: &toml::de::Error) -> String {
    match e.span() {
        Some(span) => {
            let (line, col) = line_col(src, span.start);
            format!(
                "TOML parse error at line {line}, column {col}: {}",
                e.message()
            )
        }
        None => e.message().to_owned(),
    }
}

/// 1-based (line, column) of a byte offset within `src`.
fn line_col(src: &str, byte: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i >= byte {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// [`load_toml_file`] plus that layer's `[[version_overrides]]`. Use for
/// weepcode config files; use [`load_toml_file`] directly for unrelated TOML.
pub fn load_config_file(path: &Path) -> std::io::Result<toml::Value> {
    let mut v = load_toml_file(path)?;
    apply_version_overrides_with_registered(&mut v)?;
    Ok(v)
}

pub fn load_from_disk() -> std::io::Result<toml::Value> {
    load_user_config_layer(user_weepcode_home().as_deref(), "config.toml")
}

/// Managed config filename, shared by the loaders in this module.
pub const MANAGED_CONFIG_FILENAME: &str = "managed_config.toml";

/// Requirements (cloud-cache) filename — the sibling server-synced artifact.
pub const REQUIREMENTS_FILENAME: &str = "requirements.toml";

pub fn load_managed_config() -> std::io::Result<toml::Value> {
    load_user_config_layer(user_weepcode_home().as_deref(), MANAGED_CONFIG_FILENAME)
}

/// Load a user-tier config layer from `<home>/<filename>`. With no resolvable
/// user home, returns an empty table rather than reading a cwd-relative
/// `.weepcode/<filename>` (the cwd-fallback would silently promote an untrusted
/// project `.weepcode` to the user tier).
fn load_user_config_layer(home: Option<&Path>, filename: &str) -> std::io::Result<toml::Value> {
    match home {
        Some(g) => load_config_file(&g.join(filename)),
        None => Ok(toml::Value::Table(toml::map::Map::new())),
    }
}

pub fn load_system_managed_config() -> std::io::Result<toml::Value> {
    let mut v = match system_config_dir() {
        Some(dir) => load_toml_file(&dir.join(MANAGED_CONFIG_FILENAME))?,
        None => toml::Value::Table(toml::map::Map::new()),
    };
    apply_version_overrides_with_registered(&mut v)?;
    Ok(v)
}

/// One managed-config layer: the parsed TOML and the file it came from.
#[derive(Debug, Clone)]
pub struct ManagedConfigLayer {
    pub value: toml::Value,
    pub path: std::path::PathBuf,
    /// `true` for the root-owned system layer (`/etc/weepcode`), derived from the
    /// load directory.
    pub is_system: bool,
}

/// All `managed_config.toml` layers in apply order (system first, user last).
/// Absent layers are skipped; unparsable layers are skipped with a warning.
/// One bad layer never drops the others.
pub fn managed_config_layers() -> Vec<ManagedConfigLayer> {
    managed_config_layers_at(
        system_config_dir().as_deref(),
        user_weepcode_home().as_deref(),
    )
}

/// [`managed_config_layers`] with explicit directories.
pub fn managed_config_layers_at(
    system_dir: Option<&Path>,
    user_home: Option<&Path>,
) -> Vec<ManagedConfigLayer> {
    let mut layers = Vec::new();
    for (dir, is_system) in [(system_dir, true), (user_home, false)] {
        let Some(path) = dir.map(|d| d.join(MANAGED_CONFIG_FILENAME)) else {
            continue;
        };
        if !path.is_file() {
            continue;
        }
        match load_config_file(&path) {
            Ok(value) => layers.push(ManagedConfigLayer {
                value,
                path,
                is_system,
            }),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "skipping managed_config.toml layer that failed to load or parse")
            }
        }
    }
    layers
}

/// Layers lowest→highest priority.
#[derive(Clone)]
pub struct ConfigLayers {
    pub system_managed: toml::Value,
    pub managed: toml::Value,
    pub user: toml::Value,
    pub user_requirements: Option<toml::Value>,
    pub system_requirements: Option<toml::Value>,
    /// macOS MDM requirements; highest requirements tier when present.
    pub mdm_requirements: Option<toml::Value>,
}

impl Default for ConfigLayers {
    fn default() -> Self {
        Self {
            system_managed: toml::Value::Table(Default::default()),
            managed: toml::Value::Table(Default::default()),
            user: toml::Value::Table(Default::default()),
            user_requirements: None,
            system_requirements: None,
            mdm_requirements: None,
        }
    }
}

impl ConfigLayers {
    pub fn load() -> std::io::Result<Self> {
        let system_managed = load_system_managed_config()?;
        let managed = load_managed_config()?;
        let user = load_from_disk()?;

        let user_requirements = load_requirements();
        let system_requirements = load_system_requirements();
        let mdm_requirements = crate::validation::mdm_requirements_value();

        Ok(Self {
            system_managed,
            managed,
            user,
            user_requirements,
            system_requirements,
            mdm_requirements,
        })
    }

    /// Layer merge only.
    pub fn effective_config_base(&self) -> toml::Value {
        let mut merged = self.system_managed.clone();
        deep_merge_toml(&mut merged, &self.managed);
        deep_merge_toml(&mut merged, &self.user);
        if let Some(req) = &self.user_requirements {
            deep_merge_toml(&mut merged, req);
        }
        if let Some(sys_req) = &self.system_requirements {
            deep_merge_toml(&mut merged, sys_req);
        }
        if let Some(mdm_req) = &self.mdm_requirements {
            deep_merge_toml(&mut merged, mdm_req);
        }
        merged
    }

    /// Disk layers only. Kept as a named alias of [`Self::effective_config_base`]
    /// so call sites that must never consult remote state stay explicit (the
    /// divergence it used to mark — the campaign overlay — is gone).
    pub fn effective_config_disk_only(&self) -> toml::Value {
        self.effective_config_base()
    }

    pub fn has_managed(&self) -> bool {
        self.managed.as_table().is_some_and(|t| !t.is_empty())
            || self
                .system_managed
                .as_table()
                .is_some_and(|t| !t.is_empty())
    }

    pub fn has_system_managed(&self) -> bool {
        self.system_managed
            .as_table()
            .is_some_and(|t| !t.is_empty())
    }
}

/// Disk layers only. The name mirrors the [`ConfigLayers::effective_config_disk_only`]
/// method so the (historical) divergence from the shell's loader stays explicit
/// at every call site.
pub fn load_effective_config_disk_only() -> std::io::Result<toml::Value> {
    Ok(ConfigLayers::load()?.effective_config_disk_only())
}

/// Applies matching `[[version_overrides]]` patches against the running
/// CLI version; strips the section either way. If the installed version
/// can't be parsed (broken `WEEPCODE_TEST_VERSION` in dev), silently strips
/// without applying — keeps the CLI usable on a bad dev override.
pub fn apply_version_overrides_with_registered(value: &mut toml::Value) -> std::io::Result<()> {
    match weepcode_version::installed_semver() {
        Ok(version) => apply_version_overrides(value, &version)
            .map_err(|e| std::io::Error::other(e.to_string())),
        Err(_) => {
            if let Some(table) = value.as_table_mut() {
                table.remove(version_overrides::VERSION_OVERRIDES_KEY);
            }
            Ok(())
        }
    }
}

/// Recursively merge `overrides` into `base`. Values in `overrides` win.
pub fn deep_merge_toml(base: &mut toml::Value, overrides: &toml::Value) {
    if let toml::Value::Table(overrides_table) = overrides
        && let toml::Value::Table(base_table) = base
    {
        for (key, value) in overrides_table {
            if let Some(existing) = base_table.get_mut(key) {
                deep_merge_toml(existing, value);
            } else {
                base_table.insert(key.clone(), value.clone());
            }
        }
    } else {
        *base = overrides.clone();
    }
}

/// Expand `$VAR` / `${VAR}` in all string values.
pub fn expand_env_vars_in_toml(value: &mut toml::Value) {
    match value {
        toml::Value::String(s) => {
            let expanded = expand_env_vars_in_string(s);
            if expanded != *s {
                *s = expanded;
            }
        }
        toml::Value::Array(items) => {
            for item in items {
                expand_env_vars_in_toml(item);
            }
        }
        toml::Value::Table(table) => {
            for (_, item) in table.iter_mut() {
                expand_env_vars_in_toml(item);
            }
        }
        _ => {}
    }
}

/// Expand `$VAR` / `${VAR}` in a single string.
pub fn expand_env_vars_in_string(input: &str) -> String {
    let context = |name: &str| std::env::var(name).ok();
    shellexpand::env_with_context_no_errors(input, context).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_layer_precedence_requirements_over_config_over_managed() {
        let system_managed: toml::Value =
            toml::from_str("[telemetry]\nmode = \"system_managed_value\"\n").unwrap();
        let managed: toml::Value =
            toml::from_str("[telemetry]\nmode = \"managed_value\"\n").unwrap();
        let user: toml::Value = toml::from_str("[telemetry]\nmode = \"user_value\"\n").unwrap();
        let user_requirements: toml::Value =
            toml::from_str("[telemetry]\nmode = \"user_requirements_value\"\n").unwrap();
        let system_requirements: toml::Value =
            toml::from_str("[telemetry]\nmode = \"system_requirements_value\"\n").unwrap();

        let mut merged = system_managed;
        deep_merge_toml(&mut merged, &managed);
        deep_merge_toml(&mut merged, &user);
        deep_merge_toml(&mut merged, &user_requirements);
        deep_merge_toml(&mut merged, &system_requirements);

        assert_eq!(
            merged["telemetry"]["mode"].as_str(),
            Some("system_requirements_value")
        );
    }

    #[test]
    fn effective_config_mdm_requirements_win_over_system_and_user() {
        // MDM is merged last, so an admin-forced value clamps the effective
        // config over both the user config and the system requirements layer.
        let layers = ConfigLayers {
            user: toml::from_str("[features]\nweb_fetch = true\n").unwrap(),
            system_requirements: Some(toml::from_str("[features]\nweb_fetch = true\n").unwrap()),
            mdm_requirements: Some(toml::from_str("[features]\nweb_fetch = false\n").unwrap()),
            ..Default::default()
        };
        assert_eq!(
            layers.effective_config_disk_only()["features"]["web_fetch"].as_bool(),
            Some(false),
        );
    }

    #[test]
    fn full_precedence_holds_when_values_come_from_version_overrides() {
        let cli_version = semver::Version::parse("1.8.0").unwrap();

        let mut managed: toml::Value = toml::from_str(
            r#"
            [[version_overrides]]
            minimum_version = "1.0.0"
            [version_overrides.telemetry]
            mode = "managed_versioned"
            "#,
        )
        .unwrap();
        let mut user: toml::Value = toml::from_str(
            r#"
            [[version_overrides]]
            minimum_version = "1.0.0"
            [version_overrides.telemetry]
            mode = "user_versioned"
            "#,
        )
        .unwrap();
        let mut requirements: toml::Value = toml::from_str(
            r#"
            [[version_overrides]]
            minimum_version = "1.0.0"
            [version_overrides.telemetry]
            mode = "requirements_versioned"
            "#,
        )
        .unwrap();

        apply_version_overrides(&mut managed, &cli_version).unwrap();
        apply_version_overrides(&mut user, &cli_version).unwrap();
        apply_version_overrides(&mut requirements, &cli_version).unwrap();

        let mut merged = managed;
        deep_merge_toml(&mut merged, &user);
        deep_merge_toml(&mut merged, &requirements);

        assert_eq!(
            merged["telemetry"]["mode"].as_str(),
            Some("requirements_versioned")
        );
    }

    /// Direct contract for `deep_merge_toml`: nested tables merge (siblings
    /// preserved), arrays replace (not concatenate), missing keys insert.
    #[test]
    fn deep_merge_toml_table_merge_array_replace_and_insert() {
        let mut base: toml::Value = toml::from_str(
            r#"
            [features.telemetry]
            enabled = false
            sample_rate = 0.0

            [server]
            allowed = ["a", "b"]
            "#,
        )
        .unwrap();
        let overrides: toml::Value = toml::from_str(
            r#"
            [features.telemetry]
            enabled = true

            [server]
            allowed = ["c"]

            [brand_new]
            x = 1
            "#,
        )
        .unwrap();

        deep_merge_toml(&mut base, &overrides);

        assert_eq!(
            base["features"]["telemetry"]["enabled"].as_bool(),
            Some(true)
        );
        assert_eq!(
            base["features"]["telemetry"]["sample_rate"].as_float(),
            Some(0.0)
        );
        let arr: Vec<_> = base["server"]["allowed"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert_eq!(arr, vec!["c"]);
        assert_eq!(base["brand_new"]["x"].as_integer(), Some(1));
    }

    #[test]
    fn user_version_overrides_dont_escape_their_layer() {
        let cli_version = semver::Version::parse("1.8.0").unwrap();
        let mut user: toml::Value = toml::from_str(
            r#"
            [[version_overrides]]
            minimum_version = "1.0.0"
            [version_overrides.telemetry]
            mode = "enabled"
            "#,
        )
        .unwrap();
        apply_version_overrides(&mut user, &cli_version).unwrap();
        assert_eq!(user["telemetry"]["mode"].as_str(), Some("enabled"));

        let requirements: toml::Value = toml::from_str(
            r#"
            [telemetry]
            mode = "disabled"
            "#,
        )
        .unwrap();

        let mut merged = user;
        deep_merge_toml(&mut merged, &requirements);
        assert_eq!(merged["telemetry"]["mode"].as_str(), Some("disabled"));
    }

    #[test]
    fn load_user_config_layer_is_empty_without_user_home() {
        // No resolvable user home: no user layer, and crucially no
        // cwd-relative .weepcode read.
        let v = load_user_config_layer(None, "config.toml").unwrap();
        assert_eq!(v.as_table().map(|t| t.is_empty()), Some(true));
    }

    #[test]
    fn load_user_config_layer_reads_file_when_home_present() {
        use std::io::Write;

        let dir = std::env::temp_dir().join(format!("weepcode-load-layer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("config.toml")).unwrap();
        writeln!(f, "[telemetry]\nmode = \"from_file\"\n").unwrap();

        let v = load_user_config_layer(Some(&dir), "config.toml").unwrap();
        assert_eq!(v["telemetry"]["mode"].as_str(), Some("from_file"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The returned error keeps the parser's kind + location but never the source
    /// snippet, which can carry a secret and would reach a client caller.
    #[test]
    fn parse_error_keeps_kind_but_not_snippet() {
        let dir = std::env::temp_dir().join(format!("weepcode-toml-leak-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.toml");
        // Duplicate key: the message names the key; the secret-bearing source line is only in Display.
        std::fs::write(
            &path,
            "api_key = \"weepcode-secretmustnotleak\"\napi_key = \"weepcode-secretmustnotleak2\"\n",
        )
        .unwrap();

        let msg = load_toml_file(&path).unwrap_err().to_string();
        assert!(
            msg.contains("TOML parse error at line 2"),
            "want location: {msg}"
        );
        assert!(msg.contains("duplicate key"), "want parser kind: {msg}");
        assert!(
            !msg.contains("weepcode-secretmustnotleak"),
            "leaked the secret value: {msg}"
        );
        assert!(
            !msg.contains('|') && !msg.contains('^'),
            "leaked the source snippet/caret: {msg}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
