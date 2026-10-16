# GPUI Grid

**基于 Rust + GPUI 构建的原生高性能数据表格组件，以及 AG Grid Quartz 风格的可交互网页预览。**

> This is an independent, AG Grid-inspired implementation. AG Grid is a trademark of its respective owner; this project does not include AG Grid source code, AG Grid Enterprise components, or a commercial license.

## 项目结构

```text
crates/gpui-grid-core/     纯 Rust 表格数据模型：多列排序 / 过滤 / 分组 / 聚合 / 原子编辑 / CSV
crates/gpui-grid/          原生 GPUI 组件适配器（GPUI Kit 0.7.3 DataTable）
examples/native/           GPUI 桌面组件演示应用
web/                       零运行时依赖的完整交互式预览网站
.github/workflows/ci.yml   Rust core + native GPUI (macOS) + browser test/build
.github/workflows/pages.yml GitHub Pages 自动构建与发布
web/dist/standalone.html   单文件离线预览（构建产生）
```

## 网页预览 / Web preview

```bash
cd web
npm test
npm run build
npm run dev
# Open http://127.0.0.1:4173
```

The website requires **no external NPM dependencies**. You can also open `web/dist/standalone.html` directly after building.

### 预览中的交互功能

- AG Grid Quartz-inspired columns, density, light/dark themes, hover/focus/highlight states
- Virtual scrolling over 1,000 / 12,480 / **100,000** local rows (only visible rows are mounted)
- Sort by clicking column headers; Shift+click to add multi-sort; hover for column menus
- Resize via column dividers, drag headers to reorder, pin left, hide/show columns
- Search across rows, status/region/company filters; tabs for filtered views
- Group by Region/Status/Category/Channel; collapse groups and show revenue aggregates
- Cell focus and keyboard movement, Shift-click rectangular selection, row checkboxes
- Double click or Enter to edit, typed validation, Ctrl/Cmd+C, paste tab-separated cells
- CSV import (max 20k rows in demo) and CSV export; spreadsheet formula injection guard
- Responsive layout, color themes, lightweight status and performance indicators

## Rust + GPUI 原生组件

Requires a recent stable Rust toolchain and desktop prerequisites for [GPUI Kit](https://gpui-kit.com).

```bash
# Pure data core (works without GUI)
cargo test -p gpui-grid-core

# Native GUI app — macOS recommended
cargo run -p gpui-grid-native

# Compile reusable library
cargo check -p gpui-grid
```

The native component intentionally **reuses GPUI Kit DataTable**, rather than rewriting its scrolling/rendering internals. The `gpui-grid-core` model stores typed rows and provides stable row indices to the native delegate.

### Minimal native integration

```rust
use gpui_grid::{GridDelegate, sample_orders};
use gpui_kit::*;
use gpui_kit::component::table::{DataTable, TableState};

// Within gpui_kit::open_window's |window, cx| factory:
let table = cx.new(|cx| {
    TableState::new(GridDelegate::new(sample_orders(12_480)), window, cx)
        .cell_selectable(true)
        .row_selectable(true)
});
let grid = DataTable::new(&table).bordered(true);
```

## GitHub 仓库、Actions 与 Pages 发布

- **代码仓库**：[caovc/gpui-grid](https://github.com/caovc/gpui-grid)
- **持续集成**：`.github/workflows/ci.yml` 在 Ubuntu 上测试 Rust 核心与 Web，在 macOS 上检查 GPUI 组件和桌面示例。
- **静态预览部署**：`.github/workflows/pages.yml` 在 `main` 更新 `web/` 后执行测试、打包并发布 GitHub Pages。
- **预览地址**：https://caovc.github.io/gpui-grid/ （仅在首次 Pages 部署成功后生效）。

### 首次启用 GitHub Pages（只需一次）

打开 [仓库 Settings → Pages](https://github.com/caovc/gpui-grid/settings/pages)，在 **Build and deployment → Source** 中选择 **GitHub Actions**。工作流已采用 GitHub 官方的 `configure-pages`, `upload-pages-artifact` 和 `deploy-pages` Actions。首次启用后，如此前的 Pages workflow 因尚未启用 Pages 而失败，在 Actions 页手动 **Re-run jobs** 即可。无需设置访问令牌。

本项目的浏览器预览为独立 HTML/CSS/JS 实现，**不是** Rust/GPUI 的 WebAssembly 编译产物；原生库与 Web 预览共享功能设计，但不共享运行时代码。

## Implementation / 功能范围

See [docs/FEATURES.md](docs/FEATURES.md) for native-vs-browser parity and known limitations. The HTML/JS site is an interaction showcase, **not a GPUI WASM build**. Rust native editing/group UI and enterprise-only AG Grid features are future work.

## License

MIT. Independent implementation; not endorsed by AG Grid.
