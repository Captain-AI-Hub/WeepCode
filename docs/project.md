# WeepCode 项目架构

> 本文档是对本仓库的深度审计结果，描述整体架构、关键数据流与改造方向。
> 最后更新：2026-07-18（初次审计）

## 1. 项目概览

WeepCode 是一个终端 AI 编码助手（TUI）， fork 自 WeepCode 的开源项目 **WeepCode Build**（`weepcode` CLI）。
当前改造目标：去除 WeepCode 强制登录与 weepcode/WeepCode 服务耦合，改造为支持多 Provider（OpenAI Responses /
OpenAI Compatible / Anthropic Messages）的通用 TUI 工具。

- 语言：Rust（edition 2024，工具链由 `rust-toolchain.toml` 锁定）
- 形态：Cargo workspace，约 80 个 crate，全部命名为 `weepcode-*` / `weepcode-*`（改名属后续阶段）
- 唯一可发布二进制：`crates/codegen/weepcode-pager-bin`（现阶段产物名 `weepcode-pager`，Phase 4 改为 `weepcode`）
- 协议中枢：TUI 与 agent 运行时之间通过 **ACP（Agent Client Protocol）** 通信，headless / stdio / leader 模式复用同一套 ACP 机制

## 2. 分层架构

```
Layer 0  组合根          weepcode-pager-bin        main()、jemalloc、沙箱、崩溃处理、子命令分发
Layer 1  TUI 应用        weepcode-pager            app/（事件循环、dispatch、effects）、views/（~60 组件）、
                                                  slash/（~70 命令）、acp/（agent 连接 + AcpUpdateTracker）、headless.rs
                         weepcode-pager-render     主题/渲染/终端原语
                         weepcode-pager-minimal    scrollback 渲染模式（IoC 接缝）
                         weepcode-ratatui-inline / weepcode-ratatui-textarea / weepcode-markdown(-core) / weepcode-mermaid
Layer 2  ACP 协议层      weepcode-acp-lib               进程内 / simplex / stdio ACP 通道
Layer 3  Agent 运行时    weepcode-shell            ★ 核心单体：MvpAgent、session actors、auth、config、MCP、插件
                         weepcode-agent            Agent 定义、系统提示词模板（templates/*.md）、工具集
                         weepcode-chat-state            会话状态 actor
                         weepcode-compaction       上下文压缩
                         weepcode-sampler          HTTP/SSE 推理客户端（三种 wire 格式）
                         weepcode-sampling-types   内部会话模型 + 各后端 wire 类型
                         weepcode-tools(-api) / weepcode-tool-{protocol,runtime,types} / weepcode-mcp / weepcode-memory
Layer 4  配置/基础设施   weepcode-config(-types) / weepcode-paths / weepcode-env / weepcode-auth / weepcode-http
                         weepcode-secrets（仅脱敏，非密钥库）/ weepcode-sandbox / weepcode-workspace* / 等
Layer 5  外围产品功能    weepcode-telemetry / weepcode-announcements
                         weepcode-plugin-marketplace / weepcode-version
                         —— 全部为 WeepCode 服务耦合点，Phase 3 处置
```

第三方 vendored：`third_party/{dagre_rust, graphlib_rust, mermaid-to-svg, ordered_hashmap}`。

## 3. 启动流程（weepcode-pager-bin/src/main.rs）

1. `main()`（main.rs:1592）：minimal-mode IoC hook → memtrace → fd 上限 → sentry → 释放用户指南文档 → 崩溃处理
2. 手工构建 tokio 多线程 runtime → `async_main()`（:1666）
3. rustls ring → `PagerArgs::parse_and_apply_cwd()` → 子命令分发（agent / mcp / login / update …）；`-p` 走 headless
4. TUI 路径：`weepcode_pager::app::run()`（app/mod.rs:440）
   - 加载有效配置 → 刷新 auth → 模型/远端设置预取
   - 会话恢复（--resume/--continue/--fork）
   - ACP 连接：`acp::connect` / `connect_via_leader`，在专用线程上 spawn `MvpAgent`
   - **启动认证门**：`acp/mod.rs:574` 读 auth_methods 首项 → `eager_auth_or_login_fallback`（:658/717/737）
   - `event_loop.rs:635-707`：若 `needs_login` → 播种 `AuthState::Pending` 并自动派发 `Action::Login`
   - `AppView::session_startup_allowed()`（app_view.rs:1103）要求 `AuthState::Done && TrustState::Done`
5. `event_loop`：biased `tokio::select!` 只做 IO 管道；路由全部委托 `AppView`

TUI 内部架构：`Action`（输入意图）→ `dispatch`（同步状态变更 → `Vec<Effect>`）→ `effects`（异步任务）→ `TaskResult` 回流 dispatch。
根组件 `AppView`；每个 agent 的视图模型 `AgentView`（app/agent_view/，约 48k 行）。

## 4. Agent 循环数据流

1. 输入 → `Action` → `Effect::SendPromptText` → ACP `PromptRequest`（app/effects/mod.rs:1053）
2. `MvpAgent::prompt`（weepcode-shell/src/agent/mvp_agent/acp_agent.rs:1986）→ session actor
3. `SessionActor::handle_prompt`（session/acp_session_impl/turn.rs:210）→ agentic loop（:1799）
   由 chat-state 构建后端无关的 `ConversationRequest`
4. `run_turn_via_sampler`（sampler_turn.rs:860）→ `SamplerHandle::submit_and_collect`
5. `weepcode-sampler` actor 发 HTTP SSE 请求；L2 流变换产出 `SamplingEvent`
6. 工具调用：`execute_tool_calls`（turn.rs:2260）→ weepcode-tools；权限请求经 ACP 回 TUI
7. 渲染：shell 发 ACP `SessionNotification` + `x.ai/*` 扩展 → `AcpUpdateTracker`（acp/tracker.rs）→ `RenderBlock` → ratatui

关键类型：`MvpAgent`、`SessionActor`、`ConversationRequest/Response`、`SamplingEvent`、`AcpUpdateTracker`、`AppView/AgentView`、`Action/Effect/TaskResult`。

## 5. 推理栈（已实现多后端，改造的地基）

**审计结论：三种 API 格式的协议层已经存在**，按模型经 `ApiBackend` 分发
（`weepcode-sampling-types/src/types.rs:1013`，serde snake_case）：

| 后端 | 标识 | 端点 | wire 类型 | 流变换 |
|---|---|---|---|---|
| OpenAI Chat Completions | `chat_completions`（默认） | `{base}/chat/completions` | 手写 types.rs | stream/chat_completions.rs |
| OpenAI Responses | `responses` | `{base}/responses` | async-openai 类型（仅借用类型定义） | stream/responses.rs |
| Anthropic Messages | `messages` | `{base}/messages` | 手写 messages.rs | stream/messages.rs |

- HTTP：裸 reqwest（非 async-openai 客户端、非 gRPC），SSE 经 eventsource-stream
- 认证方案：`AuthScheme::{Bearer, XApiKey}`（weepcode-sampler/src/config.rs:18）→ `Authorization: Bearer` / `x-api-key`
- 内部事件边界（干净、可插拔）：`SamplingEvent`（weepcode-sampler/src/events.rs:28）；`ConversationRequest/Item/Response` 为超集内部模型
- 凭证解析优先级（agent/config.rs:4305 `resolve_credentials`）：模型 `api_key`/`env_key` → session token → `WEEPCODE_API_KEY` 环境变量

### 已知缺口（Phase 2 修复）

- Messages 后端**不自动注入** `anthropic-version` 头，需在客户端补齐（当前只能靠 extra_headers 手写）
- 无一等 "provider" 概念：provider = 模型条目上的 `base_url + api_backend + auth_scheme + extra_headers` 四元组
- 每个请求带 WeepCode 追踪头（`x-weepcode-*`，client.rs:44-74）——第三方端点不应发送

## 6. 认证与强制登录现状

真正实现位于 `weepcode-shell/src/auth/`（约 22k 行），而非名字唬人的 weepcode-auth（只是 DI 接缝）。

- **认证方式**：WeepCode OAuth 2.1 PKCE 浏览器回环、RFC 8628 设备码、企业 OIDC、外部 provider 命令、API Key（`WEEPCODE_API_KEY` 或 per-model BYOK）
- **存储**：`$WEEPCODE_HOME/auth.json`（默认 `~/.weepcode/auth.json`），纯 JSON + 0600，原子写入 + flock；无 keychain
- **强制登录链路**：
  1. shell 侧 `build_auth_methods`（agent/auth_method.rs:139）按序通告方法：`weepcode.api_key` → `cached_token` → `grok.com`/`oidc`
  2. pager 侧取首项计算 `needs_login`（acp/mod.rs:574），`eager_auth_or_login_fallback` 尝试免交互认证
  3. 失败 → 欢迎屏登录菜单（views/welcome/mod.rs:686-725），`session_startup_allowed` 结构性阻断会话
  4. 服务端付费门 `enforce_weepcode_code_access`（mvp_agent/mod.rs:1767，仅对 WeepCode OAuth 用户生效）
- **关键事实**：per-model BYOK（`[model.*]` 配 api_key）今天就能绕过 WeepCode 登录——但被 TUI 方法通告逻辑挡住入口。改造核心是把这条已存在的路径变成一等公民。

## 7. 配置与持久化现状

- 格式：**TOML**。加载分层（低→高）：`/etc/weepcode/managed_config.toml` → `$WEEPCODE_HOME/managed_config.toml` → `$WEEPCODE_HOME/config.toml` → requirements.toml（签名云端缓存）→ macOS MDM
- 根目录：`$WEEPCODE_HOME` 否则 `~/.weepcode`（weepcode-config/src/paths.rs:35），无 XDG
- 主结构：`Config`（weepcode-shell/src/agent/config.rs:1265）
- **Provider 画像的事实载体**：`[model.<name>]` → `ConfigModelOverride`（agent/config.rs:3571）：
  `model`（路由 id）、`base_url`、`name`（展示名）、`api_key`、`env_key`、`api_backend`、`auth_scheme`、`extra_headers`、`context_window`、采样参数等
- 写入路径：`save_config`（util/config/persist.rs:14，原子写、拒绝覆盖不可解析文件，但**不序列化 `[model.*]`**）；
  原始 TOML upsert/delete 模式见 `util/config/mcp.rs:672/729`；typed 写入器 `util/config/settings_writes.rs`（~35 个 `set_*`）
- 模型目录：`resolve_model_list`（agent/config.rs:3130）= 内置默认 → 远端预取（`{models_list_url}`，磁盘缓存 models_cache.json）→ `[model.*]` 覆盖 → 全局默认
- TUI 选模型：`/model`、Ctrl+M picker、设置弹窗；CLI `-m`

## 8. WeepCode 服务耦合点清单（Phase 3 处置对象）

| 耦合点 | 位置 | 处置 |
|---|---|---|
| 推理/辅助代理默认端点 `cli-chat-proxy.grok.com` | weepcode-env/src/lib.rs:22 | 自定义 provider 模式下弃用 |
| 公共 API `api.x.ai/v1` | agent/config.rs:48 | 同上 |
| OAuth issuer `auth.x.ai` / accounts.x.ai | auth/config.rs:134 | Phase 7e：默认 issuer/client_id/CORS/devbox 链路已删；常量仅作 `is_weepcode_auth` 分类字面量保留 |
| 模型目录远端预取 | agent/models.rs:1971 | 自定义 endpoint 时跳过（部分已有 `has_custom_endpoint` 门） |
| 付费门 | mvp_agent/mod.rs:1767 | 非 WeepCode 认证已放行，保持 |
| 遥测 OTLP（带身份头）/ sentry | weepcode-telemetry | 默认关闭 |
| 公告 / 插件市场 | Layer 5 各 crate | 默认关闭或纯配置驱动 |
| 每请求 `x-weepcode-*` 追踪头 | sampler client.rs:44-74 | 非 WeepCode 端点不发送 |
| `X-WeepCode-Token-Auth` 等代理专用头 | agent/config.rs:4681 | 已按 URL 限定，保持 |

## 9. 品牌面（Phase 4 已处置，2026-07-18）

- CLI/二进制：clap `name = "weepcode"`；bin target `weepcode`；`--version` 输出 `weepcode 0.2.102` ✅
- 欢迎屏徽标/文案、信任警告、登录标签、项目目录询问 ✅（braille logo 为纯图形，保留待用户决策）
- 系统提示词 label 默认值 `"WeepCode"`；templates 已中性化 ✅
- 窗口标题、退出提示 `weepcode --resume`、`/docs`、`/usage`、README ✅
- crate 名 / 环境变量 `WEEPCODE_*`·`WEEPCODE_*` / 配置目录 `~/.weepcode` / ACP 线协议方法名 `x.ai/*`：
  **保持不动**（兼容优先），全量改名属 Phase 5（见 codemap）

## 10. 改造总方向（详见 docs/codemap.md）

1. 无凭证时不再弹 WeepCode 登录，改为进入 **Provider 设置**（三格式 + base_url/api_key/模型 id/展示名）
2. 设置结果持久化为 `config.toml` 的 `[model.*]` + 设为默认模型，重启即免登录
3. 自定义 provider 模式下切断全部 WeepCode 服务调用
4. 用户可见品牌替换为 WeepCode；crate/目录/环境变量维持现状，后续阶段再机械改名
