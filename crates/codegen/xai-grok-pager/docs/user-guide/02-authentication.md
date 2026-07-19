# Authentication

WeepCode talks to the LLM API you point it at — OpenAI, Anthropic, or any
OpenAI-compatible endpoint. There is no account sign-in wall: you configure an
**API provider** once, and it is persisted across restarts.

---

## Provider Setup (Default, Interactive)

On first launch (no provider configured), the welcome screen opens the
**Configure API Provider** form:

```bash
weepcode
```

The form asks for five things:

| Field | Notes |
|-------|-------|
| **Format** | `OpenAI Responses`, `OpenAI Compatible`, or `Anthropic` — the wire protocol of your endpoint |
| **Base URL** | e.g. `https://api.openai.com/v1` or `https://api.anthropic.com/v1` (pre-filled per format; any http(s) URL works, including local servers) |
| **API key** | Your provider key. Masked while typing |
| **Model id** | The exact model slug your endpoint expects (e.g. `gpt-5`, `claude-sonnet-4-5`) |
| **Display name** | How the model appears in the WeepCode model picker |

Submitting writes a `[model.<name>]` entry to `~/.grok/config.toml`, makes it
the default model, and unlocks the session immediately. The file is created
with owner-only permissions (`0600`). On later launches the saved profile is
picked up automatically — no setup, no login.

`Tab`/`↓` moves to the next field, `Shift-Tab`/`↑` back, `←`/`→` switches the
format on the Format row, `Enter` validates and saves, `Esc` cancels.

### Format → config mapping

| Form choice | `api_backend` | Auth header |
|-------------|---------------|-------------|
| OpenAI Responses | `responses` | `Authorization: Bearer …` |
| OpenAI Compatible | `chat_completions` | `Authorization: Bearer …` |
| Anthropic | `messages` | `x-api-key: …` + `anthropic-version: 2023-06-01` |

---

## Manual Configuration

Everything the form does can be written by hand:

```toml
# ~/.grok/config.toml
[model.my-provider]
model       = "gpt-5"                      # model id your endpoint expects
name        = "My OpenAI"                  # display name in the picker
base_url    = "https://api.openai.com/v1"
api_key     = "sk-..."
api_backend = "responses"                  # responses | chat_completions | messages

[models]
default = "my-provider"
```

Anthropic adds an auth scheme (the `anthropic-version` header is sent
automatically for the `messages` backend):

```toml
[model.claude]
model       = "claude-sonnet-4-5"
name        = "Claude"
base_url    = "https://api.anthropic.com/v1"
api_key     = "sk-ant-..."
api_backend = "messages"
auth_scheme = "x_api_key"
```

Keep secrets out of the config file with `env_key` instead of `api_key`:

```toml
[model.my-provider]
env_key = ["OPENAI_API_KEY", "WORK_OPENAI_KEY"]   # first set variable wins
```

Running `weepcode login` prints this same configuration guidance (there is no
browser sign-in). `weepcode logout` clears `~/.grok/auth.json` if one exists.

---

## Environment-Only Setup

For CI or one-off runs without a config file:

```bash
export XAI_API_KEY="sk-..."
weepcode
```

The variable is honored as a fallback credential; its historical name is kept
for compatibility and works with any OpenAI-compatible endpoint configured via
`[model.*]` or the default endpoint.

---

## Auth Precedence

Credentials resolve per request, highest to lowest:

1. **Per-model `api_key` or `env_key`** — set under `[model.<name>]` in `config.toml`. Wins whenever present.
2. **Active session token** — from `~/.grok/auth.json` (enterprise flows below).
3. **`XAI_API_KEY`** — fallback when nothing else applies.

---

## Enterprise: OIDC SSO

Pinned enterprise single sign-on (Authorization Code + PKCE against your own
IdP) is still supported for managed deployments:

```toml
# ~/.grok/config.toml
[grok_com_config.oidc]
issuer = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"

[auth]
preferred_method = "oidc"
```

With `preferred_method = "oidc"` the CLI advertises only the OIDC login
(a loopback browser flow) plus any cached session, mirroring the upstream
behavior. Without the pin, the provider setup form is the only interactive
entry.

## Enterprise: External Auth Provider

A deployment can also delegate credential minting to a local binary:
stdout carries the token (bare string or JSON with
`access_token`/`refresh_token`/`expires_in`/`issuer`), stderr carries
user-facing status. Configure with `[auth] auth_provider_command` or
`GROK_AUTH_PROVIDER_COMMAND`. Tokens refresh by re-running the binary with
`GROK_AUTH_EXPIRED=1`.

---

## Hot Reload

Changes to `~/.grok/config.toml` are picked up automatically — new
`[model.*]` entries appear in the model picker without a restart.

---

## Troubleshooting

### Debug logging

In the TUI, file logging defaults to `DEBUG`; set `GROK_LOG_FILE` to choose the
path. In headless mode (`-p`), logs go to stderr and `RUST_LOG` defaults to
`off` — set `RUST_LOG=error` (or broader) to see them.

### Common fixes

- **"Authentication failed" / 401** — Check the key and the format: Anthropic
  endpoints need the `anthropic` format (`messages` backend), OpenAI endpoints
  one of the OpenAI formats. A mismatch usually surfaces as 400/404, not 401.
- **404 model not found** — The `Model id` doesn't exist on your endpoint;
  check the provider's model list.
- **Nothing happens on submit** — Open `~/.grok/config.toml` and confirm the
  `[model.*]` entry was written; delete it and re-run the form if fields were
  mistyped.
