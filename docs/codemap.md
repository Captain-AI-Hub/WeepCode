# WeepCode 编码计划（codemap.md）

> 本文件是当前改造的**唯一权威计划**。阶段必须按序执行；每阶段末尾的**硬门禁全部通过**，
> 并在 `docs/process/{当天日期}.md` 留下验证记录后，才允许进入下一阶段。
> 最后更新：2026-07-18（初版）

## 总目标

1. 去除 xAI 强制登录：无任何凭证时，TUI 进入 **Provider 设置** 而非 xAI OAuth 登录
2. Provider 设置支持三种 API 格式：`openai-responses` / `openai-compatible` / `anthropic`，
   字段：base_url、api_key、模型 id、模型展示名称；配置**持久化**（重启免登录）
3. 自定义 Provider 模式下切断全部 xAI 服务调用（遥测/更新/公告/目录拉取/付费门等）
4. 用户可见品牌替换为 **WeepCode**，成为通用 TUI 工具

## 范围约束（用户已确认）

- 产品名：**WeepCode**
- 深度：**功能优先**。crate 名（xai-grok-*）、env 变量（GROK_*/XAI_*）、配置目录（~/.grok）
  本期不动；全量机械改名属 Phase 5（后续，未排期）

## 术语

- **Provider 画像** = `config.toml` 中一个 `[model.<name>]` 表：
  `model`(模型 id) + `name`(展示名) + `base_url` + `api_key` + `api_backend` + `auth_scheme` + `extra_headers`
- **格式映射**：`openai-responses`→`api_backend="responses"`；
  `openai-compatible`→`"chat_completions"`；`anthropic`→`"messages"` + `auth_scheme="x_api_key"` + `anthropic-version` 头

---

## Phase 0 — 文档与规则奠基

- [x] docs/project.md（架构审计）
- [x] docs/rule.md（项目规则：命名 + 进度更新）
- [x] AGENTS.md（agent 入口规则）
- [x] docs/codemap.md（本文件）
- [x] docs/process/2026-07-18.md（当日进度建档）

**硬门禁 G0**：四个文档存在且互相引用一致；AGENTS.md 位于仓库根。✅ 已通过（2026-07-18）

---

## Phase 1 — 去除强制登录

目标：干净环境（无 ~/.grok、无 env key）启动 TUI 时，不出现 xAI 登录墙；
欢迎屏提供「配置 API Provider」入口；完成配置前不阻断界面浏览，发起会话需先有可用 Provider。

- [x] shell 侧：`xai-grok-shell/src/agent/auth_method.rs` 新增非交互方法 `provider.setup`（暂用此 id，
      ACP 扩展方法命名同步去 x.ai 前缀），在无 xAI 凭证且无 BYOK 模型时作为首选项通告
- [x] pager 侧：识别 `provider.setup`（实现落在 `dispatch/auth.rs` 的 `dispatch_login`，acp/mod.rs 无需改动），
      不再播种 xAI `AuthState::Pending` 登录流，改为打开 Provider 设置界面（Phase 2 实现，本阶段可先落占位视图）
- [x] 欢迎屏登录菜单（`views/welcome/mod.rs:686-725`）移除 grok.com/OIDC 入口，替换为「配置 API Provider」
- [x] `session_startup_allowed`（app/app_view.rs:1103）：保存 Provider 后经 authenticate(xai.api_key)→AuthState::Done 放行，无需改判定
- [x] headless 路径（`xai-grok-pager/src/headless.rs:529-557`）报错文案改为提示配置 Provider，不再引导 xAI login
- [x] `login` 子命令（pager-bin main.rs:1880）改为打印 Provider 配置指引

**硬门禁 G1**（全部满足才进入 Phase 2）：
1. `cargo check -p xai-grok-shell -p xai-grok-pager` 通过
2. `HOME=$(mktemp -d)` 启动 TUI：不出现 xAI 登录 URL/设备码流程；界面可见 Provider 配置入口
3. 启动过程无对 `auth.x.ai`/`accounts.x.ai`/`cli-chat-proxy.grok.com` 的网络请求（日志/tcpdump 验证）
4. 既有 auth 相关单元测试不 regress：`cargo test -p xai-grok-shell auth` 通过

---

## Phase 2 — Provider 设置界面 + 持久化

目标：TUI 内表单完成三格式 Provider 配置并落盘；重启后免登录直接进入会话。

- [x] 新组件 `xai-grok-pager/src/views/provider_setup.rs`：
      字段 = 格式选择(responses/chat_completions/messages)、base_url、api_key、模型 id、展示名；
      非空/URL 合法性校验；anthropic 预设 base_url=`https://api.anthropic.com`
- [x] 持久化：`util/config/provider_profile.rs` upsert 模式，
      新增**语义化命名**的写入器（如 `upsert_model_override`），写入 `[model.<slug>]` 全字段
      并 `set_default_model`；含 api_key 时确保 `config.toml` 权限 0600
- [x] 配置写盘后 `reload_models_from_disk`（自 session_admin 提取）热更新模型目录 → pager 侧 authenticate(xai.api_key) →
      pager 走 eager auth → `AuthState::Done` → 会话解锁
- [x] sampler 修复：`xai-grok-sampler/src/client.rs` 当 `api_backend == Messages` 且未提供时
      自动注入 `anthropic-version: 2023-06-01`；补单元测试
- [x] xAI 追踪头收敛：`is_first_party_inference_base_url` 谓词，默认头+6 处每请求调用点全部按域收敛
- [x] `/model` 列表与 Ctrl+M picker 能选中自建 Provider 模型（[model.*] 既有目录机制，冒烟验证）
- [x] 单元测试：TOML upsert 往返、三格式字段映射、anthropic 头注入、追踪头域收敛

**硬门禁 G2**：
1. `cargo test -p xai-grok-shell -p xai-grok-sampler -p xai-grok-pager` 相关测试通过
2. 端到端手测：干净 HOME → 设置 openai-compatible（如指向本地 mock 或公开端点）→
   重启 → 无需任何登录直接进会话；`config.toml` 含完整 `[model.*]` 且权限 0600
3. 三种格式均可保存并被正确映射（读盘验证 api_backend/auth_scheme/extra_headers）
4. `cargo clippy -p` 涉及 crate 无新增 warning

---

## Phase 3 — 切断 xAI 服务耦合

目标：自定义 Provider 模式（无 xAI session）下，进程不发起任何 xAI 域请求，功能不因此崩溃。

- [x] 模型目录远端预取：无凭证时 `EndpointAuth::Session` 发请求前即 bail（remote/client.rs:739），启动零网络（lsof 实测）；无需新增门禁
- [x] 内置默认模型：grok-build `supported_in_api: false` 对 BYOK 不可见不可用（惰性保留，清理归 Phase 5）；其引发的 aux 模型 id 泄漏已在 agent_ops.rs 修复
      WeepCode 中移除或替换为中性占位（否则目录里永远挂着一个指向 xAI 代理的模型）
- [x] 遥测：无 xAI 认证时默认 Disabled（config.rs:2080），sentry 无 DSN 即 no-op；
      `GROK_TELEMETRY_ENABLED` 默认关闭
- [x] 公告（无 session 无拉取）/自更新（`should_check_for_updates` 硬禁用）/插件市场（纯配置驱动）/voice（用户触发且失败即提示）：Provider 模式下无自动流量
- [x] 付费门 `enforce_grok_code_access` 非 xAI 直通（mod.rs:1768；`is_xai_auth` 对 ApiKey 恒 false 已有 is_xai_auth_matrix 测试覆盖，不重复补）
- [x] 欢迎屏 SuperGrok 门：BYOK 无 gate 永不渲染；`/docs web` 外链与 `/usage manage` 链接已移除
- [x] workspace/leader WS relay：默认不连接（lsof 实测零连接，用户显式子命令才启用）
- [x] `resolve_inference_base_url`/`proxy_url`：mock 端到端证实推理与辅助查询只走 BYOK base_url

**硬门禁 G3**：
1. 干净 HOME + 仅配置第三方 Provider 运行完整会话，网络日志中零 xAI 域（*.x.ai / *.grok.com）请求
2. `cargo check --workspace` 通过；被禁用功能不 panic（手测公告/更新/语音入口路径）
3. 进度文件记录实际验证方式与结果

---

## Phase 4 — 用户可见品牌替换为 WeepCode

目标：用户能看到的一切 "grok/Grok Build/xAI" 字样替换或移除；内部 crate/env/目录名不动。

- [x] clap `name = "weepcode"`（app/cli.rs）；`--version` 输出 `weepcode 0.2.102`；bin target 改 `weepcode`（含 parse_and_apply_cwd 的 argv0 修正）
      （xai-grok-pager-bin/Cargo.toml，注意 default-run 与文档同步）
- [x] 欢迎屏徽标/文案、信任警告、登录标签全部 WeepCode；braille logo（纯图形无文字）保留，待用户决定是否重绘
- [x] 系统提示词 label 默认值 `"WeepCode"`（context.rs:154）；templates 三份 md 的 xAI/Grok 自述已中性化；
      templates/*.md 中 xAI/grok 自述改为中性表述；`~/.grok/docs` 指引路径说明同步
- [x] 窗口标题（title.rs 含默认/reset 与全部测试）、退出提示 `weepcode --resume`、
      `/help` 与 slash 命令帮助文案、README.md 重写
- [x] 用户可见 x.ai 链接移除（/docs web、/usage manage）；ACP 线协议方法名 `x.ai/*` 属内部协议，归 Phase 5
- [x] npm 包：package.json 改名 weepcode 并标 private（发布管线属后续事项）

**硬门禁 G4**：
1. `grep -ri "grok\|x\.ai\|xai" --include="*.rs" crates/codegen/xai-grok-pager/src crates/codegen/xai-grok-agent/templates`
   的用户可见字符串命中为零（crate 名/`use` 路径/内部标识符除外，需人工过一遍清单）
2. 构建产物 `weepcode --version` / `--help` 显示新名称；TUI 各屏无 grok 字样
3. `cargo check --workspace` 通过；既有测试无 regress

---

## Phase 5 — 全量去 grok 改名（后续，未排期）

crate（80 个 xai-grok-*）、env 变量（GROK_*/XAI_*）、配置目录（~/.grok→~/.weepcode，含迁移 shim）、
内部标识符的机械重命名。工作量大、风险高，待 Phase 1-4 稳定后单独排期，本次不执行。

---

## 执行纪律

- 任一硬门禁未通过 → 停在当前阶段修复，禁止跳阶段
- 每完成一个 checklist 项 → 立即更新 `docs/process/{当天日期}.md`（规则见 docs/rule.md §2）
- 本计划本身如需变更（阶段增删、门禁调整）→ 先改本文件并说明理由，再执行
