# Deep Research 与 Workflow

WeepCode 支持基于 Rhai 的 workflow 运行时。`deep-research` 是内置 workflow：
它会把研究目标拆成多个阶段，调度有预算上限的并行子 agent，交叉核验证据，并在完成后把带引用的报告回写到当前会话。

## 用户命令

| 命令 | 用途 |
|---|---|
| `/deep-research <query>` | 启动内置 deep-research workflow。 |
| `/workflows` | 打开 workflow 运行列表与详情视图。 |
| `/workflow <name> [args]` | 启动已注册 workflow。 |
| `/workflow pause <run>` | 暂停正在运行的 workflow。 |
| `/workflow resume <run>` | 恢复可恢复的 workflow。 |
| `/workflow stop <run>` | 停止未终止的 workflow。 |
| `/workflow save <run>` | 将可保存的运行脚本保存成可复用 workflow。 |

`<run>` 可以写 session 内唯一的 display handle，例如 `deep-research` 或 `deep-research-2`。
运行列表、任务面板、状态栏和 `/tasks` 文本块都会显示 workflow 状态；workflow 内部子 agent 不会重复出现在普通 subagent 分组里。

## Workflow 目录

Workflow 脚本是带 metadata 的 Rhai 文件，文件名必须匹配安全名称规则：
小写字母、数字和单个连字符，扩展名为 `.rhai`。

搜索顺序：

| 来源 | 目录 |
|---|---|
| 内置 | 编译进 `weepcode-shell/src/session/workflows/` |
| 项目 | `<git-root>/.weepcode/workflows/` |
| 用户 | `~/.weepcode/workflows/` |

项目目录只在当前 cwd 被信任时扫描。重复名称在同一 scope 内会报错，避免隐式覆盖。

## 运行时架构

```
/deep-research 或 workflow tool
  -> shell WorkflowRegistry 解析内置/项目/用户 Rhai 脚本
  -> WorkflowManager 创建 run_id、display handle、journal
  -> weepcode-workflow 执行 Rhai
      -> HostService 调用 agent()/parallel()
          -> SubagentCoordinator 创建 child SessionActor
          -> 子 agent 采样、工具执行、结构化输出
  -> WorkflowTracker 汇总 phase、agent、budget、status
  -> WorkflowUpdated ACP notification
  -> pager scrollback workflow block、/workflows、tasks pane、/tasks
```

Workflow 调度的 agent 调用都走现有 subagent coordinator，因此沿用当前模型配置、工具权限、工作目录、MCP、hooks、token usage 与取消机制。`agent_budget` 是绝对子 agent 调用上限；预算耗尽的运行会进入 `budget_limited`，恢复时需要更高的绝对预算。

`workflow` tool 只能从顶层会话启动。即使 `[subagents].max_depth` 或 `WEEPCODE_SUBAGENTS_MAX_DEPTH`
允许普通 `task` 子 agent 继续嵌套，workflow-spawned agent 和其他子 agent 也不能再启动 workflow，
避免后台 workflow 递归调度失控。

## 持久化

每个 workflow run 会随 session 持久化：

- `workflows/<run_id>/state.json`：运行状态、phase、agent 列表、预算、摘要等。
- `workflows/<run_id>/ack.json`：pager 已确认/清理状态。
- journal/script 副本：用于同进程恢复、完成提醒和可保存 workflow。

会话恢复时，shell 读取已持久化的 workflow run；pager 在 session reload 时合并 live 与 restored 快照，并通过 revision 去重。终止态运行仍可在 `/workflows` 中查看；清理后的 run 用 tombstone 防止旧 replay 再次插入。

## 开发入口

| 层 | 入口 |
|---|---|
| Rhai 引擎 | `crates/codegen/weepcode-workflow/` |
| 内置脚本 | `crates/codegen/weepcode-shell/src/session/workflows/deep_research.rhai` |
| registry/manager/tracker/store | `crates/codegen/weepcode-shell/src/session/workflow/` |
| slash 执行 | `crates/codegen/weepcode-shell/src/session/acp_session_impl/{slash_exec.rs,workflow.rs}` |
| workflow tool | `crates/codegen/weepcode-tools/src/implementations/weepcode_build/workflow/` |
| ACP 通知 | `crates/codegen/weepcode-shell/src/extensions/notification.rs` 和 `weepcode-pager/src/app/acp_handler/workflow_ingest.rs` |
| pager UI | `weepcode-pager/src/views/workflows.rs`、`views/tasks_pane.rs`、`app/agent_view/workflows_overlay.rs` |
