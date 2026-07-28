# WeepCode 项目地图

> 面向开发者和 AI agent 的代码导航图。`docs/project.md` 解释架构审计结论；
> 本文件补充“改哪里、看哪里、数据怎么流”的操作地图。
> 最后更新：2026-07-27。

## 1. 总览

WeepCode 是 Rust Cargo workspace，主产物是终端 TUI 二进制 `weepcode`。
运行时由两块组成：

- **pager**：用户界面、输入处理、渲染、队列、权限弹窗、会话视图。
- **shell**：agent 运行时、会话 actor、模型采样、工具执行、子 agent 调度、持久化。

两者通过 **ACP（Agent Client Protocol）** 通信。TUI 不直接调用模型，也不直接执行工具；
TUI 只发送 ACP 请求、接收 ACP 更新，并把更新渲染成 scrollback、状态栏、弹窗和 dashboard。

```
CLI main
  -> weepcode-pager app
      -> ACP channel
          -> weepcode-shell MvpAgent
              -> SessionActor
                  -> ConversationRequest
                      -> weepcode-sampler
                          -> Provider HTTP/SSE
                  -> tool calls
                      -> weepcode-tools
                      -> subagent coordinator
      <- ACP SessionNotification / ExtNotification
  <- ratatui render
```

## 2. 目录地图

| 路径 | 职责 |
|---|---|
| `crates/codegen/weepcode-pager-bin` | 二进制入口、CLI 参数、子命令、TUI 启动组合根。 |
| `crates/codegen/weepcode-pager` | 主 TUI：`app/` 状态机，`views/` 组件，`acp/` 连接和更新跟踪，`slash/` 命令。 |
| `crates/codegen/weepcode-pager-render` | 终端能力探测、主题、文本渲染、ratatui 适配。 |
| `crates/codegen/weepcode-shell` | Agent 运行时核心：`MvpAgent`、session actor、配置、认证、MCP、插件、子 agent。 |
| `crates/codegen/weepcode-agent` | Agent 定义、系统提示词、工具集构建、AGENTS.md/skills 注入。 |
| `crates/codegen/weepcode-sampler` | 后端 HTTP/SSE client，把统一请求转换成 OpenAI Responses / Chat Completions / Anthropic Messages。 |
| `crates/codegen/weepcode-sampling-types` | 后端无关会话模型与 wire 类型。 |
| `crates/codegen/weepcode-tools` | 本地工具实现、task 子 agent 工具、终端/文件系统/computer 抽象。 |
| `crates/codegen/weepcode-tools-api` | 工具 schema/codegen API，build.rs 依赖 protoc。 |
| `crates/codegen/weepcode-tool-*` | 工具协议、运行时、类型定义。 |
| `crates/codegen/weepcode-workflow` | Rhai workflow 引擎，执行脚本 op 并通过 host service 调度子 agent。 |
| `crates/codegen/weepcode-mcp` | MCP 客户端/工具接入。 |
| `crates/codegen/weepcode-config*` | 配置类型、路径、解析接缝。 |
| `crates/codegen/weepcode-auth` | 认证接缝；实际业务认证主要在 `weepcode-shell/src/auth/`。 |
| `crates/build/weepcode-proto-build` | protobuf build helper，供 codegen crates 复用。 |
| `docs/` | 项目规则、架构、计划、每日进度和用户/开发文档。 |

## 3. 启动和主循环

主入口在 `crates/codegen/weepcode-pager-bin/src/main.rs`：

1. 初始化崩溃处理、日志、runtime、终端环境。
2. 解析 CLI 参数，分发 headless、agent、mcp、add-model 等子命令。
3. TUI 路径调用 `weepcode_pager::app::run()`。
4. pager 建立 ACP 连接：本地直连走 `weepcode-pager/src/acp/mod.rs::connect`，
   leader 模式走 `connect_via_leader`。
5. ACP server 在 shell 侧创建 `MvpAgent`，`initialize()` 返回模型状态、认证方法和能力。
6. pager 进入事件循环：终端输入、ACP 消息、异步 `Effect` 结果都回到 `AppView`。

TUI 状态机的固定形态：

```
Action
  -> app/dispatch/*
      -> 同步更新 AppView / AgentView
      -> 返回 Vec<Effect>
          -> app/effects/mod.rs 异步执行
              -> TaskResult
                  -> app/dispatch/task_result.rs
```

改 TUI 交互时先看：

- `crates/codegen/weepcode-pager/src/app/actions.rs`
- `crates/codegen/weepcode-pager/src/app/dispatch/`
- `crates/codegen/weepcode-pager/src/app/effects/mod.rs`
- `crates/codegen/weepcode-pager/src/views/`
- `crates/codegen/weepcode-pager/src/acp/tracker.rs`

## 4. 会话和采样数据流

一次普通用户 prompt 的路径：

1. prompt widget 产生 `Action`，dispatch 变成 `Effect::SendPromptNow` / 相关发送 effect。
2. `app/effects/mod.rs` 构造 `acp::PromptRequest`，把 `promptId`、screen mode 等写入 `_meta`。
3. shell 的 `MvpAgent::prompt` 找到目标 `SessionHandle`，把请求送入 session actor。
4. `SessionActor::handle_prompt` 做图片规范化、计划模式状态、生命周期 hook、上下文构建。
5. `run_turn_via_sampler` 提交 `ConversationRequest` 到 `SamplerHandle::submit_and_collect`。
6. sampler 根据 `SamplerConfig.api_backend` 选择后端：
   `chat_completions`、`responses` 或 `messages`。
7. SSE 事件转换为 `SamplingEvent`，session actor 把文本、思考、工具调用增量转成 ACP 更新。
8. pager 的 `AcpUpdateTracker` 把 ACP update 规整成 scrollback block 和活动状态。

关键代码：

- `crates/codegen/weepcode-shell/src/session/acp_session.rs`
- `crates/codegen/weepcode-shell/src/session/acp_session_impl/turn.rs`
- `crates/codegen/weepcode-shell/src/session/acp_session_impl/sampler_turn.rs`
- `crates/codegen/weepcode-shell/src/session/acp_session_impl/tool_calls.rs`
- `crates/codegen/weepcode-sampler/src/client.rs`
- `crates/codegen/weepcode-sampler/src/stream/`

## 5. Agent 调度

### 5.1 顶层 agent

`MvpAgent` 是 ACP 的 agent 实现，位于 `crates/codegen/weepcode-shell/src/agent/mvp_agent/`。
它负责：

- 初始化模型目录、认证方法、客户端能力、会话清理任务。
- 创建、恢复、fork session。
- 把 ACP prompt / cancel / auth / ext_method 转发给具体 session。
- 启动子 agent coordinator。

`SessionActor` 是单个会话的执行单元。它串行处理会话命令，但会把采样、工具等待、
MCP 重启、摘要生成、后台任务等耗时操作拆到 async task 或专门 actor 中。

### 5.2 子 agent 调度

子 agent 不是 pager 直接创建的。调度链路是：

```
model tool call: task
  -> weepcode-tools TaskTool
      -> SubagentBackendResource
          -> SubagentEvent::Spawn
              -> MvpAgent subagent coordinator
                  -> handle_subagent_request
                      -> child SessionActor
                          -> child sampler/tool loop
```

核心文件：

- `crates/codegen/weepcode-tools/src/implementations/weepcode_build/task/mod.rs`
- `crates/codegen/weepcode-tools/src/implementations/weepcode_build/task/backend.rs`
- `crates/codegen/weepcode-tools/src/implementations/weepcode_build/task/types.rs`
- `crates/codegen/weepcode-shell/src/agent/subagent/mod.rs`
- `crates/codegen/weepcode-shell/src/agent/subagent/handle_request.rs`
- `crates/codegen/weepcode-shell/src/agent/mvp_agent/subagent_coordinator.rs`

调度规则：

- 顶层 session 深度为 0，默认 `MAX_SUBAGENT_DEPTH` 为 1，子 agent 默认不能再 spawn 子 agent。
- 普通 `task` 的最大嵌套深度可由 `[subagents].max_depth`、`WEEPCODE_SUBAGENTS_MAX_DEPTH`
  或 remote `subagents_max_depth` 提高；`MaxSubagentDepth` 会注入到 child agent 的 tool resources。
- blocking `task` 会等待子 agent 结果；background `task` 立即返回 task id，后续用输出查询工具轮询。
- coordinator 持有 active map、completed map、block waiters、cancel token 和 usage 状态。
- 子 agent 继承父会话的文件系统、终端 runner、hunk tracker、MCP pool、client hooks、
  session env、memory config、模型目录和部分工具配置。
- `isolation = "worktree"` 会为子 agent 创建隔离 worktree；否则共享父 cwd 或显式 cwd。
- 子 agent 完成后，coordinator 负责完成记录、usage 归集、后台任务 reparent、goal 进度通知。

### 5.3 Workflow 与 Deep Research 调度

Workflow 是 shell 侧运行时能力，不是 pager 侧脚本。入口包括：

- `/deep-research <query>`：启动内置 `deep-research` workflow。
- `/workflow <name> [args]`：启动内置、项目或用户目录中的 workflow。
- `workflow` tool：模型可启动 workflow，并通过 `/workflows`/完成提醒向用户回报进度。

调度链路：

```
/deep-research 或 workflow tool
  -> WorkflowRegistry
      -> builtin / .weepcode/workflows / ~/.weepcode/workflows
  -> WorkflowManager + WorkflowTracker + WorkflowRunStore
      -> weepcode-workflow Rhai engine
          -> WorkflowHostService
              -> agent() / parallel()
                  -> SubagentCoordinator
                      -> child SessionActor
  -> WorkflowUpdated ACP notification
      -> pager workflow_ingest
          -> scrollback WorkflowBlock
          -> /workflows overlay
          -> tasks pane / status bar / /tasks text block
```

核心文件：

- `crates/codegen/weepcode-workflow/`
- `crates/codegen/weepcode-shell/src/session/workflow/`
- `crates/codegen/weepcode-shell/src/session/workflows/deep_research.rhai`
- `crates/codegen/weepcode-shell/src/session/acp_session_impl/workflow.rs`
- `crates/codegen/weepcode-tools/src/implementations/weepcode_build/workflow/`
- `crates/codegen/weepcode-pager/src/app/acp_handler/workflow_ingest.rs`
- `crates/codegen/weepcode-pager/src/views/workflows.rs`
- `crates/codegen/weepcode-pager/src/views/tasks_pane.rs`

持久化：

- session 目录下 `workflows/<run_id>/state.json` 保存 run 状态、phase、agent roster、预算和摘要。
- `WorkflowRunStore` 负责 state/ack 写入和恢复；pager 用 revision 去重并合并恢复快照。
- 项目级脚本目录是 `<git-root>/.weepcode/workflows/`，用户级脚本目录是 `~/.weepcode/workflows/`。

调度规则：

- workflow 内部 agent 调用复用现有 subagent coordinator，因此继承模型、工具、权限、MCP、hooks 和取消逻辑。
- `workflow` tool 仍然只能从顶层会话启动，不跟随普通 `task` 的最大嵌套深度配置。
- workflow 子 agent 带 `workflow_run_id` owner；pager 会把它们归入 workflow 视图，不重复显示为普通 subagent。
- `agent_budget` 是绝对子 agent 调用上限；`budget_limited` run 只有在提高绝对预算后才能继续。

更完整的用户/开发说明见 `docs/deep-research.md`。

### 5.4 Goal 模式

Goal 模式由工具驱动，不是外部 scheduler 自动规划。相关状态在 shell session 内：

- `crates/codegen/weepcode-shell/src/session/goal_tracker.rs`
- `crates/codegen/weepcode-shell/src/session/goal_orchestrator.rs`
- `crates/codegen/weepcode-shell/src/session/goal_classifier.rs`
- `crates/codegen/weepcode-shell/src/session/goal_next_step.rs`
- `crates/codegen/weepcode-shell/src/session/templates/goal_*.md`

基本机制：

- `create_goal` / `update_goal` 工具改变 `GoalTracker` 状态。
- goal 状态通过 `GoalUpdated` ext notification 发给 pager。
- worker / verifier 轮次通常通过子 agent 或模型回合推进。
- 子 agent live token 进度走 gateway-only ephemeral update，避免 `updates.jsonl` 被高频 tick 放大。

## 6. 工具执行和权限

模型工具调用在 `SessionActor::execute_tool_calls` 中集中处理：

1. 把模型输出的 tool call 转成内部 `ToolInput`。
2. 执行 plan-mode edit gate、client hooks、权限检查。
3. 调用 `weepcode_tool_runtime` 运行具体工具。
4. 把工具开始、成功、失败、输出、权限请求等事件发回 ACP。
5. 把工具结果追加到 conversation，进入下一轮采样。

工具实现主要分三类：

- WeepCode 内置工具：`crates/codegen/weepcode-tools/src/implementations/weepcode_build/`
- OpenCode 兼容工具：`crates/codegen/weepcode-tools/src/implementations/opencode/`
- MCP 工具：由 `weepcode-mcp` 和 session MCP server 管理，名称通常带 server 前缀。

新增或删除工具时至少检查：

- 工具实现和 `ToolMetadata`。
- `weepcode-agent/src/builder.rs` 的工具集构建。
- `weepcode-tool-types` 的参数/输出类型。
- pager 的 scrollback 渲染和 snapshot 测试。
- plan mode、权限、hook、subagent depth 这些横切规则。

## 7. Provider 和模型配置

当前 Provider 以 `[model.<slug>]` 存在，没有独立 provider 表。关键字段：

- `model`：后端实际模型 id。
- `name`：TUI 展示名。
- `base_url`：推理 API base URL。
- `api_key` / `env_key`：凭证来源。
- `api_backend`：`responses`、`chat_completions`、`messages`。
- `auth_scheme`：`bearer` 或 `x_api_key`。
- `extra_headers`：如 Anthropic 的 `anthropic-version`。
- `context_window`：最大上下文窗口。

写入路径：

- pager 表单：`crates/codegen/weepcode-pager/src/views/provider_setup.rs`
- pager effect：`crates/codegen/weepcode-pager/src/app/effects/mod.rs`
- shell ext：`crates/codegen/weepcode-shell/src/extensions/provider_setup.rs`
- TOML upsert：`crates/codegen/weepcode-shell/src/util/config/provider_profile.rs`
- 模型重载：`crates/codegen/weepcode-shell/src/extensions/session_admin.rs::reload_models_from_disk`

读取和采样路径：

- 配置结构：`crates/codegen/weepcode-shell/src/agent/config.rs`
- 模型合并：`resolve_model_list`
- 凭证解析：`resolve_credentials`
- 采样配置：`sampling_config_for_model`
- HTTP client：`crates/codegen/weepcode-sampler/src/client.rs`

## 8. 持久化和运行状态

主要落盘位置：

- `$WEEPCODE_HOME/config.toml`：用户配置、模型覆盖、默认模型、MCP、UI 设置。
- `$WEEPCODE_HOME/auth.json`：认证缓存或 API key scope。
- `$WEEPCODE_HOME/sessions/`：会话更新日志、恢复、fork、搜索索引相关数据。
- 项目内 `.weepcode/`：项目级 agent 配置、hooks/plugins/skills 等。

会话恢复和导出相关代码：

- `crates/codegen/weepcode-shell/src/session/persistence.rs`
- `crates/codegen/weepcode-shell/src/session/replay_events.rs`
- `crates/codegen/weepcode-shell/src/session/fork.rs`
- `crates/codegen/weepcode-pager/src/export_cmd.rs`
- `crates/codegen/weepcode-pager/src/sessions_cmd.rs`

## 9. 常见改动入口

| 要改什么 | 先看哪里 |
|---|---|
| CLI 子命令 | `weepcode-pager-bin/src/main.rs`、`weepcode-pager/src/app/cli.rs` |
| TUI 快捷键/输入 | `weepcode-pager/src/input/`、`weepcode-pager/src/app/dispatch/` |
| 弹窗/表单 | `weepcode-pager/src/views/` |
| slash 命令 | `weepcode-pager/src/slash/` 和 shell ext methods |
| 会话主循环 | `weepcode-shell/src/session/acp_session_impl/turn.rs` |
| 工具调用 | `weepcode-shell/src/session/acp_session_impl/tool_calls.rs` |
| 新模型后端 | `weepcode-sampling-types`、`weepcode-sampler/src/client.rs`、`weepcode-sampler/src/stream/` |
| Provider 写配置 | `weepcode-shell/src/util/config/provider_profile.rs` |
| 模型列表/picker | `weepcode-shell/src/agent/config.rs`、`weepcode-pager/src/models.rs` |
| 子 agent | `weepcode-shell/src/agent/subagent/`、`weepcode-tools/.../task/` |
| Deep Research / workflow | `weepcode-workflow`、`weepcode-shell/src/session/workflow/`、`weepcode-pager/src/views/workflows.rs` |
| goal 模式 | `weepcode-shell/src/session/goal_*`、`session/templates/goal_*.md` |
| 权限/信任 | `weepcode-workspace`、`weepcode-shell/src/session/acp_session_impl/tool_calls.rs` |
| MCP | `weepcode-shell/src/session/mcp_*`、`weepcode-mcp` |
| CI | `.github/workflows/manual-ci.yml` |

## 10. 维护边界

- pager 层只处理 UI、ACP transport 和用户交互状态；不要把模型采样或工具业务塞进 pager。
- shell 层拥有 agent/session/tool orchestration；跨会话或子 agent 状态应优先放在 shell。
- sampler 只关心请求转换、HTTP/SSE、重试和事件流；不要引入 TUI 或 session 依赖。
- tools crate 只实现工具能力；需要会话上下文时通过 `SharedResources` 注入。
- 配置写入优先使用现有 TOML upsert helper，避免重新序列化丢失注释或未知字段。
- 增删 user-visible 命令、配置字段、运行模式时，同步更新 `docs/project.md`、`docs/codemap.md`
  和相关用户文档。
