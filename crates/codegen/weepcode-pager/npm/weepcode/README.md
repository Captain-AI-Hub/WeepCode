# WeepCode

Bring WeepCode into your terminal. Fast, flicker-free CLI built for plans, subagents, and parallel work.

## Install

Install from the configured npm registry:

```bash
npm i -g weepcode
```

## Get Started

```bash
# Launch the interactive TUI
weepcode

# Run a single task
weepcode -p "Explain this codebase"
```

On first launch, WeepCode opens the provider setup form. For CI or headless
environments, use an API key accepted by your configured provider:

```bash
export WEEPCODE_API_KEY="sk-..."
```

## Update

```bash
npm i -g weepcode@latest
```

## Supported Platforms

| Platform | Architecture |
|---|---|
| macOS | Apple Silicon (arm64) |
| Linux | x86_64, arm64 |
| Windows | x86_64 |

## Documentation

For full documentation including configuration, MCP servers, custom models,
headless mode, agent mode, and more, see the user guide bundled with this
repository.

## Feedback

Use the repository issue tracker or maintainer channel for feedback.
