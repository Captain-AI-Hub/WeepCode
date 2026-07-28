# AGENTS.md — WeepCode

> 面向所有 AI agent 与贡献者的入口规则。先读本文件，再按需读引用文档。

## 这是什么

WeepCode：独立开发发布的终端 AI 编码助手（Rust TUI），支持
OpenAI Responses / OpenAI Compatible / Anthropic Messages 三种 API 格式的通用工具。
架构细节见 `docs/project.md`。

## 必读文档（按序）

1. `docs/rule.md` — **项目规则，必须遵守**。核心两条：
   - **禁止意义不明的命名**：函数、文件、变量名必须见名知意，违规一律返工
   - **强制即时更新进度**：每完成一个可验证单元，立即写入 `docs/process/{当天日期}.md`
2. `docs/codemap.md` — 当前编码计划：阶段划分、checklist、**硬门禁**。
   **未达到当前阶段的硬门禁，禁止进入下一阶段。**
3. `docs/project.md` — 架构地图。动手前先确认你改的模块在图中的位置与依赖方向。

## 构建与验证

```sh
export PROTOC=$PWD/.tools/protoc/bin/protoc   # 可选覆盖；构建也会自动发现该路径
cargo check -p weepcode-pager-bin        # 快速验证（主二进制）
cargo check -p weepcode-pager -p weepcode-shell -p weepcode-sampler
cargo test -p <改动的crate>               # 改动涉及 crate 的测试
cargo clippy -p <改动的crate>             # 遵循 workspace lints
cargo fmt                                 # rustfmt.toml 已配置
cargo run -p weepcode-pager-bin          # 构建并启动 TUI（产物 target/debug/weepcode）
```

注意：包名为 `weepcode-pager-bin`，二进制产物名为 `weepcode`（Phase 5 已完成全量改名：
crate `weepcode-*`、env `WEEPCODE_*`、配置目录 `~/.weepcode`、ACP 方法名 `weepcode/*`；
无兼容 shim）。

## 工作流纪律

- 一次只做 codemap 当前阶段的事；做完跑门禁命令，把结果写进当天 `docs/process/` 文件
- 最小改动，遵循现有代码惯例，不顺手重构无关代码
- 改架构/配置/命令 → 同步更新 `docs/project.md`、`docs/codemap.md` 及用户文档
- 不擅自 git 提交/推送；不删除不认识的文件
- 使用新依赖前先确认 workspace 已依赖，否则先向用户说明

## 当前阶段提示

当前阶段以 `docs/codemap.md` 为准；改架构、配置、命令或 CI 时必须同步文档与当天进度。
