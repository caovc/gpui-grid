## 关联 Issue
Closes #

## 本次修改
<!-- 说明 core / native GPUI / WASM 的实际变更，不要把 Core-only 当成 UI-ready -->

## 实际跨端验证
- [ ] `cargo test -p gpui-grid-core --all-targets`
- [ ] macOS: `cargo check -p gpui-grid -p gpui-grid-native`
- [ ] `npm --prefix web test`
- [ ] `./scripts/build-wasm.sh --release`（真实 GPUI WebPlatform）
- [ ] `npm --prefix web run build`，确认 `gpui_grid_web_bg.wasm`
- [ ] 真实浏览器中打开 GPUI canvas，验证交互和浏览器 Console
- [ ] 如果修改了编辑、选区、排序/过滤：验证 Row ID / Column Key 在虚拟滚动、重排下仍然稳定
- [ ] 添加 Rust 单元测试、必要的 GPUI/浏览器交互测试
- [ ] 更新 `docs/FEATURES.md` 与 `docs/ROADMAP.md`（如范围变化）

## 证据与风险
**浏览器/系统：**  
**交互测试步骤：**  
**截图或录屏（特别是 WASM）：**  
**性能数据（涉及 10k/100k/宽表时）：**  
**未覆盖场景及原因：**  

> 本项目 Web 页必须加载真实 Rust/GPUI WASM；不能以独立 JS 表格作为替代或降级，除非另开专门设计审批。
