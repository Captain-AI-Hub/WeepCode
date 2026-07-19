# WeepCode

**WeepCode** 是一个终端 AI 编码助手（TUI）。它理解你的代码库、编辑文件、执行 shell
命令、管理长任务——以交互式 TUI、headless 脚本模式，或经 Agent Client Protocol（ACP）
嵌入编辑器的方式工作。

本项目 fork 自 xAI 的 Grok Build，已移除 xAI 强制登录与全部 xAI 服务耦合，
改造为**自带 Provider 的通用工具**：启动时配置一次 API Provider 即可永久使用。

## 支持的 API 格式

| 格式 | 端点 | 适用 |
|------|------|------|
| OpenAI Responses | `{base_url}/responses` | OpenAI Responses API |
| OpenAI Compatible | `{base_url}/chat/completions` | OpenAI 及各类兼容服务（含本地服务） |
| Anthropic Messages | `{base_url}/messages` | Anthropic Claude |

每种 Provider 配置包含：base_url、api_key、模型 id、展示名称，持久化在
`~/.grok/config.toml` 的 `[model.<name>]` 表中（文件权限 0600），重启免配置。

## 快速开始

```sh
cargo run -p xai-grok-pager-bin    # 构建并启动 TUI（产物名 weepcode）
```

首次启动进入 **Configure API Provider** 表单：

1. 选择格式（OpenAI Responses / OpenAI Compatible / Anthropic）
2. 填入 base_url（按格式预填，可改）
3. 填入 api_key（掩码输入）
4. 填入模型 id 与展示名称
5. Enter 保存——写入 `~/.grok/config.toml` 并直接进入会话

也可以手写配置：

```toml
# ~/.grok/config.toml
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

## 构建要求

- **Rust**：工具链由 `rust-toolchain.toml` 锁定
- **protoc**：proto codegen 需要；放在 `PATH` 上或设置 `PROTOC` 环境变量。
  （上游的 DotSlash 方式仍兼容：安装 dotslash 后会自动使用 `bin/protoc`）

```sh
export PROTOC=$PWD/.tools/protoc/bin/protoc   # 如使用本地 protoc
cargo build -p xai-grok-pager-bin              # 产物：target/debug/weepcode
cargo check -p xai-grok-pager-bin              # 快速验证
```

## 仓库文档

- `AGENTS.md` — agent 入口规则（命名、进度、门禁纪律）
- `docs/project.md` — 整体架构审计
- `docs/codemap.md` — 改造计划：阶段、checklist、硬门禁
- `docs/rule.md` — 项目规则
- `docs/process/` — 每日进度记录

## 与上游（Grok Build）的差异

- 无 xAI OAuth / grok.com 登录；Provider 设置取而代之
- 无遥测上报、无自更新通道、无公告/付费门（无 xAI 凭证时全部不激活）
- `x-grok-*` 等 xAI 追踪头只发往一方端点，第三方 Provider 收不到
- crate 名（`xai-grok-*`）、环境变量（`GROK_*`/`XAI_*`）、配置目录（`~/.grok`）
  暂时保持兼容，全量改名见 `docs/codemap.md` Phase 5

## License

Apache-2.0（与上游一致），见 `LICENSE` 与 `THIRD-PARTY-NOTICES`。
