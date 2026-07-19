//! `weepcode/provider/*` extension handlers (WeepCode).
//!
//! The pager's provider setup form submits `weepcode/provider/save`; the agent
//! persists the profile as a `[model.<slug>]` table in `~/.grok/config.toml`,
//! points `[models].default` at it, and hot-reloads the model catalog so a
//! follow-up `authenticate("xai.api_key")` succeeds on BYOK credentials alone
//! — no xAI OAuth flow is ever involved.

use agent_client_protocol as acp;
use serde::Deserialize;

use super::{ExtResult, parse_params, to_raw_response};
use crate::agent::MvpAgent;
use crate::util::config::{ProviderApiFormat, ProviderProfileInput, upsert_provider_profile};

/// ACP extension method the pager's provider setup form submits to.
pub const PROVIDER_SAVE_METHOD: &str = "weepcode/provider/save";

pub async fn handle(agent: &MvpAgent, args: &acp::ExtRequest) -> ExtResult {
    match args.method.as_ref() {
        PROVIDER_SAVE_METHOD => handle_save_provider(agent, args),
        _ => Err(acp::Error::method_not_found()),
    }
}

#[derive(Deserialize)]
struct SaveProviderParams {
    format: String,
    base_url: String,
    api_key: String,
    model_id: String,
    display_name: String,
    context_window: Option<u64>,
}

fn handle_save_provider(agent: &MvpAgent, args: &acp::ExtRequest) -> ExtResult {
    let params: SaveProviderParams = parse_params(args)?;
    let format = ProviderApiFormat::from_wire_str(&params.format).ok_or_else(|| {
        acp::Error::invalid_params().data(format!(
            "unknown format {:?}; expected openai-responses | openai-compatible | anthropic",
            params.format
        ))
    })?;
    let profile = ProviderProfileInput {
        format,
        base_url: params.base_url,
        api_key: params.api_key,
        model_id: params.model_id,
        display_name: params.display_name,
        context_window: params.context_window,
    }
    .validated()
    .map_err(|e| acp::Error::invalid_params().data(e))?;

    let slug = upsert_provider_profile(&profile)
        .map_err(|e| acp::Error::internal_error().data(format!("failed to persist provider: {e:#}")))?;

    // Rebuild the live catalog from the just-written config so the new entry
    // is selectable and its BYOK credentials unlock `authenticate(xai.api_key)`.
    super::session_admin::reload_models_from_disk(agent)?;

    tracing::info!(slug = %slug, model_id = %profile.model_id, "provider profile saved");
    to_raw_response(&serde_json::json!({
        "ok": true,
        "slug": slug,
        "default_model": slug,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_provider_params_deserialize() {
        let raw = r#"{
            "format": "anthropic",
            "base_url": "https://api.anthropic.com/v1",
            "api_key": "sk-ant",
            "model_id": "claude-sonnet-4-5",
            "display_name": "Claude"
        }"#;
        let params: SaveProviderParams = serde_json::from_str(raw).unwrap();
        assert_eq!(params.format, "anthropic");
        assert_eq!(params.model_id, "claude-sonnet-4-5");
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(ProviderApiFormat::from_wire_str("grok").is_none());
    }
}
