//! Persist API-provider profiles (`[model.<slug>]` tables) in `~/.weepcode/config.toml`.
//!
//! WeepCode: a "provider profile" is the bundle the in-TUI provider setup form
//! collects — API format, base URL, API key, model id, display name. On disk
//! it is a plain `[model.*]` override plus `[models].default`, byte-for-byte
//! what a hand-written BYOK entry looks like, so every existing resolver
//! (`resolve_model_list`, `resolve_credentials`, the sampler) keeps working
//! unchanged.

use anyhow::{Context, Result, bail};
use toml::Value as TomlValue;
use toml::map::Map as TomlMap;

use super::mcp::user_config_path;

/// `anthropic-version` header value sent for the Anthropic Messages format.
/// Matches the sampler-side auto-injection default.
pub const ANTHROPIC_VERSION_HEADER: &str = "2023-06-01";

/// API formats offered by the provider setup form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderApiFormat {
    /// OpenAI Responses API (`POST {base_url}/responses`).
    OpenAiResponses,
    /// OpenAI-compatible Chat Completions (`POST {base_url}/chat/completions`).
    OpenAiCompatible,
    /// Anthropic Messages API (`POST {base_url}/messages`).
    Anthropic,
}

impl ProviderApiFormat {
    /// Parse the wire value sent by the pager's provider setup form.
    pub fn from_wire_str(s: &str) -> Option<Self> {
        match s {
            "openai-responses" => Some(Self::OpenAiResponses),
            "openai-compatible" => Some(Self::OpenAiCompatible),
            "anthropic" => Some(Self::Anthropic),
            _ => None,
        }
    }

    /// `[model.*] api_backend` value this format maps to.
    pub fn api_backend(self) -> &'static str {
        match self {
            Self::OpenAiResponses => "responses",
            Self::OpenAiCompatible => "chat_completions",
            Self::Anthropic => "messages",
        }
    }
}

/// A provider profile as received from the setup form (not yet validated).
#[derive(Clone, Debug)]
pub struct ProviderProfileInput {
    pub format: ProviderApiFormat,
    pub base_url: String,
    pub api_key: String,
    pub model_id: String,
    pub display_name: String,
    /// Max context window in tokens; `None` keeps the resolver default.
    pub context_window: Option<u64>,
}

impl ProviderProfileInput {
    /// Trim all fields and reject empty / malformed ones.
    ///
    /// `base_url` must be an absolute http(s) URL; the trailing slash is
    /// stripped so the sampler's `{base}/chat/completions`-style joins stay
    /// clean. `context_window` must be positive when present.
    pub fn validated(self) -> Result<Self, String> {
        let trimmed = Self {
            format: self.format,
            base_url: self.base_url.trim().trim_end_matches('/').to_string(),
            api_key: self.api_key.trim().to_string(),
            model_id: self.model_id.trim().to_string(),
            display_name: self.display_name.trim().to_string(),
            context_window: self.context_window,
        };
        if trimmed.base_url.is_empty() {
            return Err("base_url is required".to_string());
        }
        match url::Url::parse(&trimmed.base_url) {
            Ok(u)
                if matches!(u.scheme(), "http" | "https")
                    && !u.host_str().unwrap_or("").is_empty() => {}
            _ => return Err("base_url must be a valid http(s) URL".to_string()),
        }
        if trimmed.api_key.is_empty() {
            return Err("api_key is required".to_string());
        }
        if trimmed.model_id.is_empty() {
            return Err("model_id is required".to_string());
        }
        if trimmed.display_name.is_empty() {
            return Err("display_name is required".to_string());
        }
        if matches!(trimmed.context_window, Some(0)) {
            return Err("context_window must be positive".to_string());
        }
        Ok(trimmed)
    }
}

/// Derive a stable `[model.<slug>]` table key from the display name.
///
/// Lowercases, maps every non-alphanumeric run to a single `-`, trims leading
/// and trailing dashes. Falls back to `custom-provider` when nothing usable
/// remains (e.g. an all-CJK display name).
pub fn provider_profile_slug(display_name: &str) -> String {
    let mut slug = String::with_capacity(display_name.len());
    let mut last_was_dash = false;
    for ch in display_name.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let slug = slug.trim_end_matches('-').to_string();
    if slug.is_empty() {
        "custom-provider".to_string()
    } else {
        slug
    }
}

/// Build the TOML table for one `[model.<slug>]` entry.
fn provider_profile_table(profile: &ProviderProfileInput) -> TomlValue {
    let mut table = TomlMap::new();
    table.insert(
        "model".to_string(),
        TomlValue::String(profile.model_id.clone()),
    );
    table.insert(
        "name".to_string(),
        TomlValue::String(profile.display_name.clone()),
    );
    table.insert(
        "base_url".to_string(),
        TomlValue::String(profile.base_url.clone()),
    );
    table.insert(
        "api_key".to_string(),
        TomlValue::String(profile.api_key.clone()),
    );
    table.insert(
        "api_backend".to_string(),
        TomlValue::String(profile.format.api_backend().to_string()),
    );
    if let Some(context_window) = profile.context_window {
        table.insert(
            "context_window".to_string(),
            TomlValue::Integer(context_window as i64),
        );
    }
    if profile.format == ProviderApiFormat::Anthropic {
        table.insert(
            "auth_scheme".to_string(),
            TomlValue::String("x_api_key".to_string()),
        );
        let mut headers = TomlMap::new();
        headers.insert(
            "anthropic-version".to_string(),
            TomlValue::String(ANTHROPIC_VERSION_HEADER.to_string()),
        );
        table.insert("extra_headers".to_string(), TomlValue::Table(headers));
    }
    TomlValue::Table(table)
}

/// Upsert `profile` as `[model.<slug>]` in `~/.weepcode/config.toml` and point
/// `[models].default` at it. Returns the slug used.
///
/// The write is atomic (tmp + rename) and the file is tightened to
/// owner-only permissions afterwards because it now carries an API key.
/// A slug collision with an existing `[model.*]` entry gets a `-2`, `-3`, …
/// suffix rather than silently replacing the user's hand-written entry.
///
/// The input is (re)validated here so the persistence boundary never writes
/// untrimmed or malformed fields regardless of the caller.
pub fn upsert_provider_profile(profile: &ProviderProfileInput) -> Result<String> {
    upsert_provider_profile_at(&user_config_path(), profile)
}

/// Same as [`upsert_provider_profile`] but targets an explicit config file;
/// kept separate so tests never touch the real `~/.weepcode`.
pub fn upsert_provider_profile_at(
    path: &std::path::Path,
    profile: &ProviderProfileInput,
) -> Result<String> {
    let profile = &profile.clone().validated().map_err(anyhow::Error::msg)?;
    let mut root: TomlValue = match std::fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).with_context(|| {
            format!(
                "refusing to overwrite unparseable config {}",
                path.display()
            )
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => TomlValue::Table(TomlMap::new()),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    let root_table = root
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("config root is not a table"))?;

    let models_override = root_table
        .entry("model")
        .or_insert_with(|| TomlValue::Table(TomlMap::new()))
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("[model] is not a table"))?;

    let base_slug = provider_profile_slug(&profile.display_name);
    let mut slug = base_slug.clone();
    let mut suffix = 2u32;
    while models_override.contains_key(&slug) {
        slug = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
    models_override.insert(slug.clone(), provider_profile_table(profile));

    let models_section = root_table
        .entry("models")
        .or_insert_with(|| TomlValue::Table(TomlMap::new()))
        .as_table_mut()
        .ok_or_else(|| anyhow::anyhow!("[models] is not a table"))?;
    models_section.insert("default".to_string(), TomlValue::String(slug.clone()));

    let toml_str = toml::to_string_pretty(&root)?;
    let tmp_path = path.with_extension("toml.tmp");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp_path, &toml_str)?;
    std::fs::rename(&tmp_path, path)?;
    crate::util::secure_file::ensure_owner_only_permissions(path)
        .with_context(|| format!("failed to tighten permissions on {}", path.display()))?;
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input(format: ProviderApiFormat) -> ProviderProfileInput {
        ProviderProfileInput {
            format,
            base_url: "https://api.openai.com/v1/".to_string(),
            api_key: "sk-test".to_string(),
            model_id: "gpt-5".to_string(),
            display_name: "My OpenAI".to_string(),
            context_window: None,
        }
    }

    #[test]
    fn wire_str_round_trip() {
        assert_eq!(
            ProviderApiFormat::from_wire_str("openai-responses"),
            Some(ProviderApiFormat::OpenAiResponses)
        );
        assert_eq!(
            ProviderApiFormat::from_wire_str("openai-compatible"),
            Some(ProviderApiFormat::OpenAiCompatible)
        );
        assert_eq!(
            ProviderApiFormat::from_wire_str("anthropic"),
            Some(ProviderApiFormat::Anthropic)
        );
        assert_eq!(ProviderApiFormat::from_wire_str("weepcode"), None);
    }

    #[test]
    fn validation_trims_and_strips_trailing_slash() {
        let input = ProviderProfileInput {
            base_url: "  https://api.openai.com/v1/  ".to_string(),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        let validated = input.validated().unwrap();
        assert_eq!(validated.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn validation_rejects_bad_url_and_empty_fields() {
        let bad_url = ProviderProfileInput {
            base_url: "not-a-url".to_string(),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        assert!(bad_url.validated().is_err());
        let empty_key = ProviderProfileInput {
            api_key: "   ".to_string(),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        assert!(empty_key.validated().is_err());
        let empty_model = ProviderProfileInput {
            model_id: String::new(),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        assert!(empty_model.validated().is_err());
        let ftp = ProviderProfileInput {
            base_url: "ftp://example.com/v1".to_string(),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        assert!(ftp.validated().is_err());
    }

    #[test]
    fn slug_derivation() {
        assert_eq!(provider_profile_slug("My OpenAI"), "my-openai");
        assert_eq!(provider_profile_slug("  Claude 4!! "), "claude-4");
        assert_eq!(provider_profile_slug("---"), "custom-provider");
        assert_eq!(provider_profile_slug("千问"), "custom-provider");
        assert_eq!(provider_profile_slug("a__b"), "a-b");
    }

    #[test]
    fn upsert_writes_full_profile_and_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let slug =
            upsert_provider_profile_at(&path, &sample_input(ProviderApiFormat::OpenAiResponses))
                .unwrap();
        assert_eq!(slug, "my-openai");

        let parsed: TomlValue = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let entry = &parsed["model"]["my-openai"];
        assert_eq!(entry["model"].as_str(), Some("gpt-5"));
        assert_eq!(entry["name"].as_str(), Some("My OpenAI"));
        assert_eq!(
            entry["base_url"].as_str(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(entry["api_key"].as_str(), Some("sk-test"));
        assert_eq!(entry["api_backend"].as_str(), Some("responses"));
        assert!(entry.get("auth_scheme").is_none());
        assert_eq!(parsed["models"]["default"].as_str(), Some("my-openai"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "config with api_key must be 0600");
        }
    }

    #[test]
    fn upsert_writes_context_window_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let input = ProviderProfileInput {
            context_window: Some(128_000),
            ..sample_input(ProviderApiFormat::OpenAiCompatible)
        };
        let slug = upsert_provider_profile_at(&path, &input).unwrap();
        let parsed: TomlValue = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            parsed["model"][&slug]["context_window"].as_integer(),
            Some(128_000)
        );

        // Absent → field omitted, resolver default (200k) applies instead.
        let dir2 = tempfile::tempdir().unwrap();
        let path2 = dir2.path().join("config.toml");
        let slug2 =
            upsert_provider_profile_at(&path2, &sample_input(ProviderApiFormat::OpenAiCompatible))
                .unwrap();
        let parsed2: TomlValue = toml::from_str(&std::fs::read_to_string(&path2).unwrap()).unwrap();
        assert!(parsed2["model"][&slug2].get("context_window").is_none());
    }

    #[test]
    fn validation_rejects_zero_context_window() {
        let input = ProviderProfileInput {
            context_window: Some(0),
            ..sample_input(ProviderApiFormat::OpenAiResponses)
        };
        assert!(input.validated().is_err());
    }

    #[test]
    fn upsert_anthropic_maps_backend_and_headers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let input = ProviderProfileInput {
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_key: "sk-ant".to_string(),
            model_id: "claude-sonnet-4-5".to_string(),
            display_name: "Claude".to_string(),
            format: ProviderApiFormat::Anthropic,
            context_window: None,
        };
        let slug = upsert_provider_profile_at(&path, &input).unwrap();
        let parsed: TomlValue = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let entry = &parsed["model"][&slug];
        assert_eq!(entry["api_backend"].as_str(), Some("messages"));
        assert_eq!(entry["auth_scheme"].as_str(), Some("x_api_key"));
        assert_eq!(
            entry["extra_headers"]["anthropic-version"].as_str(),
            Some(ANTHROPIC_VERSION_HEADER)
        );
    }

    #[test]
    fn upsert_collision_gets_numeric_suffix_and_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[model.my-openai]\nmodel = \"hand-written\"\n").unwrap();

        let slug =
            upsert_provider_profile_at(&path, &sample_input(ProviderApiFormat::OpenAiCompatible))
                .unwrap();
        assert_eq!(slug, "my-openai-2");

        let parsed: TomlValue = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            parsed["model"]["my-openai"]["model"].as_str(),
            Some("hand-written"),
            "existing hand-written entry must survive"
        );
        assert_eq!(
            parsed["model"]["my-openai-2"]["api_backend"].as_str(),
            Some("chat_completions")
        );
        assert_eq!(parsed["models"]["default"].as_str(), Some("my-openai-2"));
    }

    #[test]
    fn upsert_refuses_to_clobber_unparseable_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is [ not toml").unwrap();
        let result =
            upsert_provider_profile_at(&path, &sample_input(ProviderApiFormat::OpenAiResponses));
        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "this is [ not toml"
        );
    }
}
