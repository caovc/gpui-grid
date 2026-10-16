# GPUI Grid Development Roadmap

> 更新时间：2026-10-17。代码和 CI 状态以 `main` 为准。开发计划不是已经完成的功能，也不表示后台自动执行。
>
> **Epic：** [#1](https://github.com/caovc/gpui-grid/issues/1) · [所有 Issues](https://github.com/caovc/gpui-grid/issues) · [功能实况表](FEATURES.md)

## 1. 技术目标与边界

构建 **AG Grid 风格、由 Rust 实现、由 GPUI 渲染** 的可复用表格组件库。桌面和浏览器同用 `gpui_grid::GridShowcase` / `GridDelegate` 与纯 Rust `gpui-grid-core`；浏览器运行 **实际的 GPUI WebPlatform WASM**，而不是 HTML/JS 模拟表格。

现有基础（代码已有 ≠ GPUI 界面可用）：

| 层 | 已实现 | 仍缺失 |
|---|---|---|
| `gpui-grid-core` | typed Value/Row/Column、稳定 row_id、sort/filter/edit/group/CSV 引擎 | 复杂 Filter AST、增量查询、RowModel 接口 |
| `gpui-grid` | DataTable delegate、基础 cell/row select、单列排序、列 resize/move、简单过滤按钮 | 编辑器、区域选择模型、clipboard、多列 UI、分组行、主题配置 |
| `gpui-grid-web` | `wasm32-unknown-unknown` + wasm-bindgen + GPUI WebPlatform，Rust 共享视图 | 实际浏览器自动化 smoke/输入法和 GPU 兼容性矩阵 |
| `web` | GPUI Canvas/WASM 加载页、Pages 发布 | 组件级 E2E 截图回归与可靠错误证据 |

2026-10-17 查验：最新两条 GitHub Actions 的 **Rust core、macOS native、WASM build、Pages publish jobs 均通过**；但目前没有声称浏览器 GPU/点击交互自动化已验证。

## 2. 实施阶段与 GitHub 工单

按 **依赖关系 + PR 可独立合并 + WASM 可观察效果** 安排，避免一次大改难以验证。

| 阶段 | 推荐顺序 | 交付验收点 | 粗略工作量* |
|---|---|---|---|
| **M0 / v0.2 foundation** | [#2](https://github.com/caovc/gpui-grid/issues/2) 真 WASM 浏览器 Smoke → [#3](https://github.com/caovc/gpui-grid/issues/3) GridState / GridOptions / Events | GPU 画面与最小交互可证、稳定 Row/Column ID、可扩展 API | 1–3 周 |
| **M1 / v0.2 interaction** | [#4](https://github.com/caovc/gpui-grid/issues/4) 编辑器、[#5](https://github.com/caovc/gpui-grid/issues/5) 区域选择 → [#6](https://github.com/caovc/gpui-grid/issues/6) Clipboard/Undo | 输入/校验/撤销/复制粘贴在 Rust 桌面与 WASM 同功能 | 3–6 周 |
| **M2 / v0.3 data & UX** | [#7](https://github.com/caovc/gpui-grid/issues/7) 类型筛选 ∥ [#8](https://github.com/caovc/gpui-grid/issues/8) 多列排序 ∥ [#9](https://github.com/caovc/gpui-grid/issues/9) 列管理/Quartz 主题 | 表头控件全部对应真实 Rust GridState | 3–5 周 |
| **M3 / v0.4 advanced & perf** | [#10](https://github.com/caovc/gpui-grid/issues/10) 分组聚合 → [#11](https://github.com/caovc/gpui-grid/issues/11) 10 万行/200 列基准及优化 | 高级行模型无数据错误，附跨端可复现性能数据 | 3–6 周 |
| **M4 / v0.5 extensibility** | [#12](https://github.com/caovc/gpui-grid/issues/12) Async/ServerSide RowModel、[#13](https://github.com/caovc/gpui-grid/issues/13) CSV/XLSX、[#14](https://github.com/caovc/gpui-grid/issues/14) crate API / docs、[#15](https://github.com/caovc/gpui-grid/issues/15) IME/Accessibility | 远程数据、导入导出、跨端发布与 API 文档 | 4–8 周 |
| **Future** | [#16](https://github.com/caovc/gpui-grid/issues/16) Pivot / Tree / Merge / Fill Handle RFC | RFC + 原型，择优启动，不纳入首版完成标准 | 待评估 |

`*` 时间仅为**熟悉 Rust + GPUI 的开发者**的粗略工程估计，含测试与返工空间；GPUI 实验性 WebAssembly 后端、输入法与浏览器 GPU 可显著改变周期。不是固定截止日或自动运行计划。可并行项目用 `∥` 标记。

### 首个迭代应当这样做

1. 先开 **#2**：真实 GPU/Canvas 的浏览器 E2E Smoke，CI 留截图、console/network 错误；证明确实是 GPUI WASM。
2. 并行 **#3**：把 `GridState`, `ColumnDef`, `GridApi`, `GridEvent` 作为统一模型，避免之后的编辑/过滤/选区状态各自维护。
3. 然后 **#4 + #5**：编辑器和稳定 ID 的选区模型；完成后才能正确支持 **#6** 批量复制粘贴。
4. 保持每个 PR 有真实 WASM 的可观察变更，避免将 Core-only 特性标注为浏览器 UI 支持。
5. 每个阶段结束时执行性能/交互回归，并更新 `docs/FEATURES.md` 与本文件。

## 3. 统一验收标准（Definition of Done）

所有功能 Issue/PR 都必须包含：

- [ ] 功能在 **Rust core / GPUI 原生组件**完成，WASM 通过相同 Rust UI 渲染；无 HTML/JS 列表替身。
- [ ] `cargo test -p gpui-grid-core --all-targets` 通过；新增的 Rust 业务/状态逻辑有边界测试。
- [ ] macOS `cargo check -p gpui-grid -p gpui-grid-native` 通过。
- [ ] `./scripts/build-wasm.sh --release`、`npm --prefix web test`、`npm --prefix web run build` 成功并验证确实产出 `gpui_grid_web_bg.wasm`。
- [ ] 浏览器能加载 GPUI Canvas；交互行为由真实 Rust 状态驱动。#2 的 E2E 具备可用后接入。
- [ ] 正确处理 stable row_id / column_key，筛选/排序/列移动后无错误编辑及选中串行。
- [ ] 对性能敏感的功能提供行数/列数、操作时间、浏览器和运行环境，避免未经测试声称「百万行不卡顿」。
- [ ] 更新 [FEATURES.md](FEATURES.md)、样例和用户可见交互说明；非支持项如实标注。

## 4. 开发与合并约定

- 一条 Issue 对应一个功能分支和一个可 review 的 PR：`feat/<issue-number>-<slug>`。
- 推荐每个 PR 以 **核心模型与测试 → GPUI UI → WASM smoke → 文档** 为四个原子验收点。
- 核心引擎不要依赖 GUI 平台。UI 使用 `row_id + column_key` 定位操作，不依赖虚拟视口索引。
- 原生和 WASM 保持同一个 `GridShowcase`；Web 页面只承担 WASM 加载、容器和错误提示。
- 若依赖上游 GPUI Kit API 更改，先以 `cargo check` / WASM 构建验证再更新锁版本。
- 未完成 CI 不合并主分支；将浏览器渲染不确定性与已验证编译分开表述。

## 5. 风险与优先级约束

1. **WASM GPU 可用性**：首选基于上游 WebPlatform，浏览器兼容性需要真实 Canvas/E2E 证据。
2. **原生 UI 状态模型**：编辑、选区、列排序一旦和滚动索引混淆，后期返工代价高，因此 #3 优先。
3. **IME/Clipboard/Accessibility**：不同平台差异大，先确保端到端可用，再标注平台支持。
4. **性能和内存**：12,480 行当前可演示并不意味着 100k/1M 行已有性能保证。
5. **许可**：独立 AG Grid-inspired 实现，不引用 AG Grid 源码或 Enterprise 受许可模块。

## 6. 日常跟踪方式

- 看 [Epic #1](https://github.com/caovc/gpui-grid/issues/1) 的检查清单了解整体完成度。
- 每实现一项功能，在对应 Issue 的 PR 描述中引用 `Closes #N`，记录截图/测试/基准。
- 优先级 `P0`＝阻断后续开发，`P1`＝核心可用性，`P2`＝高级/发布，`P3`＝后续增强。
- 只依据合并到 `main` 并通过测试的代码更新功能矩阵，不能依据 Issue 创建或 PR 提交就宣布功能完成。
