# WeepCode

**WeepCode** 是一个终端 AI 编码助手（TUI）。它理解你的代码库、编辑文件、执行 shell
命令、管理长任务——以交互式 TUI、headless 脚本模式，或经 Agent Client Protocol（ACP）
嵌入编辑器的方式工作。

本项目作为 **WeepCode** 独立开发发布，已移除上游强制登录与专属服务耦合，
是**自带 Provider 的通用工具**：启动时配置一次 API Provider 即可永久使用。

## 支持的 API 格式

| 格式 | 端点 | 适用 |
|------|------|------|
| OpenAI Responses | `{base_url}/responses` | OpenAI Responses API |
| OpenAI Compatible | `{base_url}/chat/completions` | OpenAI 及各类兼容服务（含本地服务） |
| Anthropic Messages | `{base_url}/messages` | Anthropic Claude |

每种 Provider 配置包含：base_url、api_key、模型 id、展示名称，持久化在
`~/.weepcode/config.toml` 的 `[model.<name>]` 表中（文件权限 0600），重启免配置。

## 安装

Linux（x86_64 / aarch64）与 macOS Apple Silicon：

```sh
curl -fsSL https://github.com/Captain-AI-Hub/WeepCode/releases/latest/download/install.sh | sh
```

Windows x86_64（PowerShell）：

```powershell
irm https://github.com/Captain-AI-Hub/WeepCode/releases/latest/download/install.ps1 | iex
```

Unix 默认安装到 `~/.local/bin`，Windows 默认安装到
`%LOCALAPPDATA%\Programs\WeepCode\bin` 并写入用户 `PATH`。可通过
`WEEPCODE_INSTALL_DIR` 更改目录，或通过 `WEEPCODE_VERSION` 安装指定 tag。安装器会在解压前依据
Release 中的 `SHA256SUMS` 校验安装包。

## 快速开始

```sh
weepcode
```

首次启动进入 **Configure API Provider** 表单：

1. 选择格式（OpenAI Responses / OpenAI Compatible / Anthropic）
2. 填入 base_url（按格式预填，可改）
3. 填入 api_key（掩码输入）
4. 填入模型 id 与展示名称
5. Enter 保存——写入 `~/.weepcode/config.toml` 并直接进入会话

也可以手写配置：

```toml
# ~/.weepcode/config.toml
[model.claude]
model       = "claude-sonnet-4-5"
name        = "Claude"
base_url    = "https://api.anthropic.com/v1"
api_key     = "sk-ant-..."
api_backend = "messages"
auth_scheme = "x_api_key"

[models]
default = "claude"
```

## Deep Research

TUI 内可用 `/deep-research <query>` 启动内置深度研究 workflow。运行后用
`/workflows` 查看阶段、子 agent、预算和结果；也可用 `/workflow pause|resume|stop <run>`
管理运行。可复用 workflow 放在项目 `.weepcode/workflows/` 或用户
`~/.weepcode/workflows/`。更多细节见 `docs/deep-research.md`。

## 构建要求

- **Rust**：工具链由 `rust-toolchain.toml` 锁定
- **protoc**：proto codegen 需要；构建会依次检查 `PROTOC`、仓库内
  `.tools/protoc/bin/protoc`、DotSlash 的 `bin/protoc` 和 `PATH`。

```sh
cargo build -p weepcode-pager-bin   # 产物：target/debug/weepcode
cargo check -p weepcode-pager-bin   # 快速验证
```

## 仓库文档

- `AGENTS.md` — agent 入口规则（命名、进度、门禁纪律）
- `docs/project.md` — 整体架构审计
- `docs/deep-research.md` — Deep Research / workflow 用户与开发说明
- `docs/codemap.md` — 改造计划：阶段、checklist、硬门禁
- `docs/rule.md` — 项目规则
- `docs/process/` — 每日进度记录

## 独立开发状态

- 无上游 OAuth 登录；Provider 设置取而代之
- 无默认遥测上报、无自更新通道、无公告/付费门
- 一方追踪头仅发往一方端点，第三方 Provider 收不到
- Phase 5 已完成全量改名：crate `weepcode-*`、环境变量 `WEEPCODE_*`、
  配置目录 `~/.weepcode`、ACP 方法名 `weepcode/*`；无兼容 shim

## License

Apache-2.0（与上游一致），见 `LICENSE` 与 `THIRD-PARTY-NOTICES`。
