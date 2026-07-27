# WeepCode 编码计划（codemap.md）

> 本文件是当前改造的**唯一权威计划**。阶段必须按序执行；每阶段末尾的**硬门禁全部通过**，
> 并在 `docs/process/{当天日期}.md` 留下验证记录后，才允许进入下一阶段。
> 最后更新：2026-07-18（初版）

## 总目标

1. 去除 WeepCode 强制登录：无任何凭证时，TUI 进入 **Provider 设置** 而非 WeepCode OAuth 登录
2. Provider 设置支持三种 API 格式：`openai-responses` / `openai-compatible` / `anthropic`，
   字段：base_url、api_key、模型 id、模型展示名称；配置**持久化**（重启免登录）
3. 自定义 Provider 模式下切断全部 WeepCode 服务调用（遥测/更新/公告/目录拉取/付费门等）
4. 用户可见品牌替换为 **WeepCode**，成为通用 TUI 工具

## 范围约束（用户已确认）

- 产品名：**WeepCode**
- 深度：**功能优先**。crate 名（weepcode-*）、env 变量（WEEPCODE_*/WEEPCODE_*）、配置目录（~/.weepcode）
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

目标：干净环境（无 ~/.weepcode、无 env key）启动 TUI 时，不出现 WeepCode 登录墙；
欢迎屏提供「配置 API Provider」入口；完成配置前不阻断界面浏览，发起会话需先有可用 Provider。

- [x] shell 侧：`weepcode-shell/src/agent/auth_method.rs` 新增非交互方法 `provider.setup`，
      ACP 扩展方法命名同步使用 `weepcode/*` 前缀，在无 WeepCode 凭证且无 BYOK 模型时作为首选项通告
- [x] pager 侧：识别 `provider.setup`（实现落在 `dispatch/auth.rs` 的 `dispatch_login`，acp/mod.rs 无需改动），
      不再播种 WeepCode `AuthState::Pending` 登录流，改为打开 Provider 设置界面（Phase 2 实现，本阶段可先落占位视图）
- [x] 欢迎屏登录菜单（`views/welcome/mod.rs:686-725`）移除上游 OAuth/OIDC 入口，替换为「配置 API Provider」
- [x] `session_startup_allowed`（app/app_view.rs:1103）：保存 Provider 后经 authenticate(weepcode.api_key)→AuthState::Done 放行，无需改判定
- [x] headless 路径（`weepcode-pager/src/headless.rs:529-557`）报错文案改为提示配置 Provider，不再引导 WeepCode login
- [x] `login` 子命令（pager-bin main.rs:1880）改为打印 Provider 配置指引

**硬门禁 G1**（全部满足才进入 Phase 2）：
1. `cargo check -p weepcode-shell -p weepcode-pager` 通过
2. `HOME=$(mktemp -d)` 启动 TUI：不出现 WeepCode 登录 URL/设备码流程；界面可见 Provider 配置入口
3. 启动过程无对上游认证/代理服务的网络请求（日志/tcpdump 验证）
4. 既有 auth 相关单元测试不 regress：`cargo test -p weepcode-shell auth` 通过

---

## Phase 2 — Provider 设置界面 + 持久化

目标：TUI 内表单完成三格式 Provider 配置并落盘；重启后免登录直接进入会话。

- [x] 新组件 `weepcode-pager/src/views/provider_setup.rs`：
      字段 = 格式选择(responses/chat_completions/messages)、base_url、api_key、模型 id、展示名；
      非空/URL 合法性校验；anthropic 预设 base_url=`https://api.anthropic.com`
- [x] 持久化：`util/config/provider_profile.rs` upsert 模式，
      新增**语义化命名**的写入器（如 `upsert_model_override`），写入 `[model.<slug>]` 全字段
      并 `set_default_model`；含 api_key 时确保 `config.toml` 权限 0600
- [x] 配置写盘后 `reload_models_from_disk`（自 session_admin 提取）热更新模型目录 → pager 侧 authenticate(weepcode.api_key) →
      pager 走 eager auth → `AuthState::Done` → 会话解锁
- [x] sampler 修复：`weepcode-sampler/src/client.rs` 当 `api_backend == Messages` 且未提供时
      自动注入 `anthropic-version: 2023-06-01`；补单元测试
- [x] WeepCode 追踪头收敛：`is_first_party_inference_base_url` 谓词，默认头+6 处每请求调用点全部按域收敛
- [x] `/model` 列表与 Ctrl+M picker 能选中自建 Provider 模型（[model.*] 既有目录机制，冒烟验证）
- [x] 单元测试：TOML upsert 往返、三格式字段映射、anthropic 头注入、追踪头域收敛

**硬门禁 G2**：
1. `cargo test -p weepcode-shell -p weepcode-sampler -p weepcode-pager` 相关测试通过
2. 端到端手测：干净 HOME → 设置 openai-compatible（如指向本地 mock 或公开端点）→
   重启 → 无需任何登录直接进会话；`config.toml` 含完整 `[model.*]` 且权限 0600
3. 三种格式均可保存并被正确映射（读盘验证 api_backend/auth_scheme/extra_headers）
4. `cargo clippy -p` 涉及 crate 无新增 warning

---

## Phase 3 — 切断 WeepCode 服务耦合

目标：自定义 Provider 模式（无 WeepCode session）下，进程不发起任何 WeepCode 域请求，功能不因此崩溃。

- [x] 模型目录远端预取：无凭证时 `EndpointAuth::Session` 发请求前即 bail（remote/client.rs:739），启动零网络（lsof 实测）；无需新增门禁
- [x] 内置默认模型：weepcode-build `supported_in_api: false` 对 BYOK 不可见不可用（惰性保留，清理归 Phase 5）；其引发的 aux 模型 id 泄漏已在 agent_ops.rs 修复
      WeepCode 中移除或替换为中性占位（否则目录里永远挂着一个指向 WeepCode 代理的模型）
- [x] 遥测：无 WeepCode 认证时默认 Disabled（config.rs:2080），sentry 无 DSN 即 no-op；
      `WEEPCODE_TELEMETRY_ENABLED` 默认关闭
- [x] 公告（无 session 无拉取）/自更新（`should_check_for_updates` 硬禁用）/插件市场（纯配置驱动）/voice（用户触发且失败即提示）：Provider 模式下无自动流量
- [x] 付费门 `enforce_weepcode_code_access` 非 WeepCode 直通（mod.rs:1768；`is_weepcode_auth` 对 ApiKey 恒 false 已有 is_weepcode_auth_matrix 测试覆盖，不重复补）
- [x] 欢迎屏上游订阅门：BYOK 无 gate 永不渲染；`/docs web` 外链与 `/usage manage` 链接已移除
- [x] workspace/leader WS relay：默认不连接（lsof 实测零连接，用户显式子命令才启用）
- [x] `resolve_inference_base_url`/`proxy_url`：mock 端到端证实推理与辅助查询只走 BYOK base_url

**硬门禁 G3**：
1. 干净 HOME + 仅配置第三方 Provider 运行完整会话，网络日志中零上游认证/代理服务请求
2. `cargo check --workspace` 通过；被禁用功能不 panic（手测公告/更新/语音入口路径）
3. 进度文件记录实际验证方式与结果

---

## Phase 4 — 用户可见品牌替换为 WeepCode

目标：用户能看到的旧品牌字样替换或移除；内部 crate/env/目录名不动。

- [x] clap `name = "weepcode"`（app/cli.rs）；`--version` 输出 `weepcode 0.2.102`；bin target 改 `weepcode`（含 parse_and_apply_cwd 的 argv0 修正）
      （weepcode-pager-bin/Cargo.toml，注意 default-run 与文档同步）
- [x] 欢迎屏徽标/文案、信任警告、登录标签全部 WeepCode；braille logo（纯图形无文字）保留，待用户决定是否重绘
- [x] 系统提示词 label 默认值 `"WeepCode"`（context.rs:154）；templates 三份 md 的 WeepCode/WeepCode 自述已中性化；
      templates/*.md 中 WeepCode/weepcode 自述改为中性表述；`~/.weepcode/docs` 指引路径说明同步
- [x] 窗口标题（title.rs 含默认/reset 与全部测试）、退出提示 `weepcode --resume`、
      `/help` 与 slash 命令帮助文案、README.md 重写
- [x] 用户可见上游链接移除（/docs web、/usage manage）；ACP 线协议方法名归 Phase 5
- [x] npm 包：package.json 改名 weepcode 并标 private（发布管线属后续事项）

**硬门禁 G4**：
1. `grep -ri "旧品牌\|上游域名" --include="*.rs" crates/codegen/weepcode-pager/src crates/codegen/weepcode-agent/templates`
   的用户可见字符串命中为零（crate 名/`use` 路径/内部标识符除外，需人工过一遍清单）
2. 构建产物 `weepcode --version` / `--help` 显示新名称；TUI 各屏无 weepcode 字样
3. `cargo check --workspace` 通过；既有测试无 regress

---

## Phase 5 — 全量去 weepcode 改名（2026-07-19 用户拍板：立即执行，不要兼容 shim）

范围与规则（用户确认）：crate 统一为 `weepcode-*`；env 变量统一为 `WEEPCODE_*`；配置目录统一为 `~/.weepcode`，
项目级配置目录统一为 `.weepcode/` 与 `/etc/weepcode`；ACP 方法名统一为 `weepcode/*`；
auth 方法 id `weepcode.api_key` 与 proto 包 `weepcode.tools.v1` 保持当前命名；
标识符统一为 `weepcode_*` / `WeepCode*`。
**不读旧 env、不读旧目录（无兼容 shim）。** 保留项：真实服务域名常量
（属 dead 企业路径，改域名无意义反而误导）、上游订阅产品名（惰性子系统）、
third_party 注释引用同步、旧品牌文件名一并 git mv。

执行步骤：
- [x] 阶段A：72 个 crate 目录 git mv + 全部 Cargo.toml 改写（脚本 .tools/rename_phase5.py）
- [ ] 阶段B：全仓库有序替换（哨兵保护真实服务域名 → WeepCode 命名）+ 含旧品牌文件名 git mv + Cargo.lock 同步
- [x] `cargo check --workspace` 零错误
- [x] 关键 crate 测试全过；clippy 无新增警告（BuiltinAgentName 的 kebab 派生经 Weepcode 标识符统一修复；加密模板按 scripts/encrypt_templates.py 重生成）
- [x] PTY 冒烟（~/.weepcode + WEEPCODE_HOME）+ mock 端到端复跑全过
- [x] 文档同步（改名 pass 已覆盖 + 手工修漏）

**硬门禁 G5**：
1. `cargo check --workspace` 通过
2. 旧品牌 grep 命中仅剩白名单：真实服务域名常量、上游订阅产品名、SOURCE_REV/THIRD-PARTY、
   历史进度文档（process/ 为历史记录不改写）
3. 冒烟 + mock 复跑通过（配置落盘到 ~/.weepcode/config.toml）
4. 既有测试无新增失败（上游既有 8 个失败基线不变）

---

## Phase 6 — login → add-model 改名 + 表单最大上下文窗口（2026-07-19 用户提出）

目标：「login」之名已无实（不再有登录）。首次改名为 config 后与 `/settings` 的原始
`config` 别名冲突，经全命令审计最终定名 **add-model**；添加模型时可设置最大上下文窗口。

- [x] CLI 子命令 `weepcode add-model`（aliases: `login`、`config`）打印 Provider 配置指引
- [x] slash 命令 `/add-model`（alias `/login`；文件 add_model.rs）打开 Provider 表单；
      `/settings` 保留原始别名 `config`/`preferences`/`prefs`
- [x] Provider 表单新增第 6 字段「Max context (tokens)」：纯数字、预填 200000；
      链路：pager 表单 → Effect::SaveProviderProfile → `weepcode/provider/save` 参数 →
      `provider_profile` 写入 `[model.*].context_window`
- [x] 全命令审计：`weepcode setup`/`update` 禁用（上游服务），slash 硬隐藏
      /share /feedback /imagine /imagine-video /privacy，voice 硬关
- [x] 测试：表单校验（数字/空/非法/粘贴过滤）、upsert 含 context_window 往返与省略、零值拒绝

**硬门禁 G6**：
1. `weepcode add-model` 与 alias（`login`/`config`）均打印配置指引；TUI 内 `/add-model` 与 `/login` 均可用
2. `cargo test` 涉及 crate 通过；表单提交带 context_window 后 `config.toml` 正确落盘该字段
3. PTY 冒烟复跑通过（新增字段后表单键序同步更新）

---

## Phase 7 — 死代码切除（2026-07-19 用户批准开工）

目标：把审计确认的死子系统连根拔（不只是禁入口）。按风险从低到高分阶段，每阶段过编译。

- [x] 7a 隐藏 slash 命令实体删除：`share.rs`/`feedback.rs`/`privacy.rs`/`imagine.rs`/
      `imagine_video.rs` 及其注册；shell 侧对应扩展 `extensions/{feedback,share,privacy}.rs`
      与 ext_method 路由臂
      **完成（2026-07-22）**：shell 侧 feedback 子系统（session/feedback{,_manager}.rs、
      agent/feedback_client.rs、agent_ops 织入、BuiltinAction::Feedback）与
      coding_data_sharing/coding_data_retention_opt_out 全链路已删；pager 侧防御性
      hide-by-name 与 set_share_visible 已清；2 个 two_line_row 测试改用真实
      render_mermaid 行修复。详见 docs/process/2026-07-22.md。
- [x] 7b `weepcode-mixpanel` crate 摘除（含 telemetry client 中的 mixpanel 接线）
      **完成（2026-07-20）**：client.rs 去 mixpanel 接线 + sync_profile；Cargo.toml/workspace/目录
      已清；`cargo check -p weepcode-telemetry --tests` 通过。
      **补清（2026-07-27）**：删除残留目录与惰性 config 字段（mixpanel_enabled/token）、
      env/build-env 覆盖、requirements 强制项和文档说明。
- [x] 7c `weepcode-voice` crate + pager 语音 UI（voice.rs 命令、dispatch/voice、
      prompt widget 语音按钮、acp_handler voice 状态、shell 侧 voice 会话支持）摘除
      **完成（2026-07-21）**：`weepcode-voice` crate 目录/成员/依赖已清；pager 侧 voice.rs 命令、
      dispatch/voice、voice/{mod,auth,handle}、5 个 Action 变体（EnableVoiceMode/VoiceToggle/VoiceStop/
      SetVoiceCaptureMode/SetVoiceSttLanguage）+ 2 个 ActionId、VoiceState/VoiceTarget 枚举、AppView 6 个
      voice 字段及 voice_* 方法簇、event_loop voice chord/cold-start/select 臂、acp_handler voice 状态、
      prompt widget VoicePromptOverlay、dashboard 录音徽章/overlay、settings voice_capture_mode/
      voice_stt_language 条目全部移除。Shell 侧：Features/Requirements.voice_mode、resolve_voice_mode/
      is_voice_mode_enabled、pin_feature!(voice_mode)、RemoteSettings.voice_mode_enabled、
      UiConfig.voice_{capture_mode,stt_language}、set_voice_{capture_mode,stt_language}、acp_agent "voiceMode"
      meta 全部删除。死代码连带清理：client_identity::client_user_agent、glyphs::record_dot、
      settings_modal enum_choice_gated_off 的 voice 项。`cargo check --workspace` + `--tests` 0 error/0 warning；
      pager lib 7143 测试通过（剩 6 失败属 7a/7d 的 imagine/privacy/coding_data_sharing，非 voice）。
- [x] 7d billing/upsell 子系统（pager dispatch/billing.rs、credit_bar、upsell 视图、
      FetchAppBilling、/usage 命令；shell 订阅检查/付费门）摘除
      **完成（2026-07-22）**：pager 侧 dispatch/billing.rs、credit_bar、app/subscription.rs、
      /usage、credit_limit 块、Effect::FetchBilling 及相关字段/测试全删；shell 侧
      extensions/billing.rs、check_subscription、付费门主体（enforce_weepcode_code_access/
      retry_subscription_check/tier_allowed 等）、agent/subscription_check.rs、
      RemoteSettings 7 个死字段、AuthMeta.gate 全删。pager lib 测试回到 8 个上游既有基线；
      shell 4 个失败均为 Phase 5 改名遗留（新基线）。保留决策点与 announcements promo CTA
      处置见 docs/process/2026-07-22.md 待办。
- [x] 7e 上游专属 OAuth 修剪：默认 client_id/issuer 常量、devbox stub、
      上游 CORS；保留通用 OIDC + 外部 provider + api_key
      **完成（2026-07-22）**：devbox 登录链路整根拔除（stub/manager/recovery/
      app/acp_agent/sampler_turn/flow，含 `run_cli_login --devbox` 与 pager 死字段）；
      `auth/config.rs` 默认 oauth2（上游 issuer + 硬编码 client_id）、
      `weepcode_oauth2_issuer()`/`use_local_auth()`/本地 issuer、上游 CORS
      全删；`auth_scope()` 改稳定占位 scope；上游 method id、`is_weepcode_auth`
      分类器常量、`LEGACY_AUTH_SCOPE` 按线协议/活逻辑/本地存储键理由保留。
      详见 docs/process/2026-07-22.md。
- [x] 7f `weepcode-update` crate 摘除：`--version` 的 channel_label 改静态后整 crate 移除
      **完成（2026-07-20）**：crate 目录/成员/依赖已清；channel_label→`weepcode_version::CHANNEL_LABEL`
      静态常量、channel_name→`CHANNEL_NAME`；pager `run()`/event_loop 去掉 bg_update_rx；main.rs 去掉
      UpdateWaitHandle/finish_update_on_exit/build_update_config/enforce_minimum_version_or_exit 等；
      `Update`/`Setup`/`Share` 死子命令一并删除（share_cmd.rs、session::share、remote client.share_session）。
      `cargo check --workspace` 通过。`--no-auto-update` 保留为兼容 no-op。
      relay/leader 的 `LeaderAutoUpdateConfig` 保留（shell 侧活类型，传 None）。
      **补清（2026-07-27）**：删除仍残留在文件系统/索引中的 `crates/codegen/weepcode-update`。

**硬门禁 G7**：
1. 每阶段 `cargo check --workspace` 通过；最终二进制编译通过且体积不增
2. `git grep` 被删路径零引用残留（crate 名/命令名/方法名）
3. 全量测试不 regress（仅上游既有 8 个 pager 失败基线不变）
4. PTY 冒烟 + mock 端到端复跑通过

---

## 执行纪律

- 任一硬门禁未通过 → 停在当前阶段修复，禁止跳阶段
- 每完成一个 checklist 项 → 立即更新 `docs/process/{当天日期}.md`（规则见 docs/rule.md §2）
- 本计划本身如需变更（阶段增删、门禁调整）→ 先改本文件并说明理由，再执行

## 运维任务 — 手动 CI

- [x] 新增 GitHub Actions 手动 CI：仅 `workflow_dispatch`，不自动响应 `push` / `pull_request`
- [x] 平台范围限定为 Linux x86_64、Linux aarch64、macOS aarch64、Windows x86_64
- [x] 手动触发时通过 `platform` 输入选择单个平台，或选择 `all` 跑全部平台
- [x] 每个平台执行 fmt/check/terminal 测试/release build，并上传对应二进制 artifact
