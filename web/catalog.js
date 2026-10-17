/** Documentation metadata only — no row data, virtual scrolling, or grid renderer. */
export const FEATURE_GROUPS = [
  {id:'start',label:'GETTING STARTED',title:'开始使用'},
  {id:'features',label:'CORE FEATURES',title:'核心特性'},
  {id:'advanced',label:'ON THE ROADMAP',title:'进阶能力'},
];

export const FEATURES = [
  {
    id:'overview',group:'start',icon:'◫',en:'Overview',title:'Data Grid 总览',short:'组件总览',
    status:'live',summary:'用 Rust 构建一套高性能 Data Grid，桌面原生与浏览器中的真实 GPUI WebAssembly 共用相同组件。',
    tags:['Rust 2024','GPUI Kit 0.7.3','WebAssembly','12,480 rows'],
    demoContext:'体验列排序、固定列、区域选择、编辑与快速过滤。操作直接更新 Rust GridModel。',
    hints:['点击列标题：按列排序','拖拽列标题或分隔线：移动、调整列宽','Shift+单击或 Shift+方向键：选择范围','双击/F2：编辑，Enter 保存'],
    description:'GPUI Grid 是独立的 Rust 数据表格组件。GridModel 负责带类型的数据和查询状态，GridDelegate 负责与 GPUI Kit DataTable 的虚拟化视口衔接，GridShowcase 是桌面应用和 WebAssembly 共用的 Rust 视图。文档页面只控制示例的路由，不渲染 HTML 表格。',
    steps:['通过 sample_orders() 或 GridOptions 创建带类型的行列模型。','用 GridDelegate 将模型映射到 GPUI Kit DataTable。','原生使用 GPUI Platform；浏览器通过 wasm-bindgen 加载同一份 Rust 视图。'],
    api:[['GridModel','带类型的行数据、排序、过滤和编辑事务'],['GridDelegate::new(model)','连接模型与 GPUI DataTable 虚拟化渲染'],['GridShowcase::new(window, cx)','构建共享的桌面 / WASM 示例视图']],
    notes:['WASM WebPlatform 目前仍是实验性/展示用途。','AG Grid 是独立产品；本项目为独立实现，不包含其商业源码。','Core 已有的功能不一定有 GPUI UI，请对照功能矩阵。'],
    code:`use gpui_kit::*;
use gpui_grid::GridShowcase;

// Native app and browser WASM render this same Rust view.
gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
    GridShowcase::new(window, cx)
}).expect("Failed to open the GPUI Grid window");`,
  },
  {
    id:'sorting',group:'features',icon:'↕',en:'Sorting',title:'行排序 Sorting',short:'排序',
    status:'live',summary:'带类型的数值排序与 GPUI 表头交互。点击 Revenue 的 Rust 操作按钮，或直接点击任何可排序的列标题。',
    tags:['Single-column UI','Numeric values','Stable sort'],
    demoContext:'点击“Highest first”或“Lowest first”；演示真正的 Rust 数值比较而非字符串排序。',
    hints:['Highest first：按收入从高到低排序','Lowest first：按收入升序排列','点击列标题：原生 GPUI 排序','Clear sort：恢复初始行顺序'],
    description:'排序逻辑位于 gpui-grid-core。列 ID 是排序配置的唯一键，数值字段使用 Value::Number 与 f64::total_cmp，避免将收入按字符串顺序排列。当前 GPUI 表头提供单列排序；Core 支持多列排序，但多列排序 UI 仍在规划中。',
    steps:['列定义指定正确的数据类型，例如 Number。','调用 GridDelegate::set_sort 或 GridModel::set_sorts。','TableState::refresh 更新 GPUI 可见行索引；稳定 row_id 不发生变化。'],
    api:[['Sort::descending("amount")','创建降序条件'],['GridDelegate::set_sort(Vec<Sort>)','在原生 GPUI 表格中应用查询排序'],['GridModel::set_sorts(Vec<Sort>)','纯 Rust Core 多列排序 API（UI 尚未接线）']],
    notes:['多列排序尚未提供原生 UI，请勿将 Core-only 能力视作已交付。','当前数据完全本地，Server-side sorting 尚未实现。'],
    code:`use gpui_grid::{GridDelegate, Sort, sample_orders};

let mut grid = GridDelegate::new(sample_orders(12_480));
// Numeric Value::Number sorting; not lexicographic strings.
grid.set_sort(vec![Sort::descending("amount")]).unwrap();
// After mutating the delegate inside TableState, call table.refresh(cx).`,
  },
  {
    id:'filtering',group:'features',icon:'⌕',en:'Filtering',title:'数据筛选 Filtering',short:'筛选',
    status:'live',summary:'使用 Rust 的类型化过滤器。尝试筛选 Revenue > $5,000，也可以点击 Completed only 查看快捷过滤。',
    tags:['Typed filter','Quick filter','Stable row IDs'],
    demoContext:'“Revenue > $5,000” 使用真正的 FilterOp::GreaterThan，而不是网页模拟结果。',
    hints:['Revenue > $5,000：数值条件过滤','Completed only：全文快捷筛选','Clear column filter：移除数值过滤','All rows：取消快捷文本过滤'],
    description:'Rust GridModel 同时维护 filters 和 quick_filter。类型化过滤比较 Value::Number；字符串快捷过滤匹配行内容。多个字段的条件在 Core 中组合，过滤后重新计算可见行索引而不是改变数据存储顺序。',
    steps:['构建 Filter 的 column、op 和 value。','调用 GridDelegate::set_filters() 更新类型条件。','选择 All rows / Clear column filter 即可独立清除对应条件。'],
    api:[['FilterOp::GreaterThan','数值大于条件'],['Filter { column, op, value }','Rust 中带类型的筛选条件'],['GridDelegate::set_filters(Vec<Filter>)','应用 GPUI 表格筛选'],['GridDelegate::set_quick_filter(str)','跨字段文本搜索']],
    notes:['本示例提供预设筛选控件；完整表头筛选弹窗仍在 #7 开发。','浏览器 JS 不处理行过滤，也不持有数据数组。'],
    code:`use gpui_grid::{
    GridDelegate, Filter, FilterOp, Value, sample_orders
};

let mut grid = GridDelegate::new(sample_orders(12_480));
grid.set_filters(vec![Filter {
    column: "amount".into(),
    op: FilterOp::GreaterThan,
    value: Value::Number(5_000.0),
}]).unwrap();`,
  },
  {
    id:'selection',group:'features',icon:'▤',en:'Cell Selection',title:'单元格选区 Cell Selection',short:'区域选择',
    status:'live',summary:'类电子表格的矩形选区：Shift+单击、Shift+方向键和 Ctrl/Cmd+A。Rust 状态保留稳定行列 ID。',
    tags:['Shift + click','Shift + arrows','Select all','Stable IDs'],
    demoContext:'选择 2×2 区域，或点击“Select all visible”一次选中当前可见的所有单元格。',
    hints:['单击单元格设置锚点','Shift+单击扩展到矩形终点','Shift+↑/↓/←/→ 扩展选区','Ctrl/Cmd+A 全选当前可见表格'],
    description:'CellRange 仅存储 anchor 与 focus（row_id + column_key）。排序、列移动和虚拟滚动时再投影为 SelectionRect，选区不会被重排后的行索引污染。即使全选 124,800 个单元格，Rust 也不需要创建 124,800 份状态。',
    steps:['单击创建 CellAddress 锚点。','按住 Shift 移动焦点或点击目标单元格。','GridDelegate 根据当前可见行列计算矩形范围，GPUI 只绘制视口内的选区高亮。'],
    api:[['CellAddress::new(row_id, column)','稳定单元格地址'],['GridModel::select_cell(cell, extend)','按锚点扩展选区'],['GridDelegate::select_all_visible()','选中当前可见行列'],['SelectionRect::cell_count()','计算当前选区大小']],
    notes:['拖拽框选与滚动自动扩展还在 #5 开发中。','Ctrl/Cmd+A 不会逐个创建选中单元格。','输入框编辑时，键盘快捷键保留正常文本编辑行为。'],
    code:`use gpui_grid::{
    CellAddress, CellRange, GridModel, sample_orders
};

let mut model = sample_orders(12_480);
model.select_cell(
    Some(CellAddress::new(1, "customer")), false
).unwrap();
model.select_cell(
    Some(CellAddress::new(2, "company")), true
).unwrap();
// Uses stable row IDs and current visible column ordering.`,
  },
  {
    id:'editing',group:'features',icon:'✎',en:'Cell Editing',title:'单元格编辑 Cell Editing',short:'单元格编辑',
    status:'live',summary:'真正的 GPUI InputState 在虚拟单元格内编辑。支持 F2、Enter、Tab、Esc、类型校验及只读字段。',
    tags:['GPUI InputState','F2 / Enter / Tab','Typed validation'],
    demoContext:'双击 Customer / Company 单元格，修改内容后按 Enter 保存，或按 Esc 取消。',
    hints:['双击可编辑单元格或选中后按 F2','Enter 保存 · Esc 取消','Tab / Shift+Tab 保存并切换字段','数值字段必须输入有限数值'],
    description:'编辑器不是 DOM 输入框的网格替身。GridDelegate 在选中位置渲染真正 GPUI InputState；输入会通过 Rust 编辑事务按稳定 row_id 写回。非法数值或只读列拒绝写入。出于 IME 输入完整性考虑，当前不会在 Blur 时自动提交。',
    steps:['双击或 F2 打开单元格里的 InputState。','Rust parse_cell_value 按列类型解析用户输入。','GridModel::apply_edits 对所有数据变更进行原子校验。'],
    api:[['InputState','GPUI 内置输入编辑状态'],['parse_cell_value(DataType, str)','文字转换为类型化 Value'],['GridDelegate::edit(&[Edit])','按稳定行 ID 提交编辑事务']],
    notes:['IME 中文输入及更多失焦策略仍在 #4 测试中。','编辑失败保留草稿，不会写入部分数据。'],
    code:`use gpui_grid::{GridDelegate, Edit, Value, sample_orders};

let mut grid = GridDelegate::new(sample_orders(12_480));
grid.edit(&[Edit {
    row_id: 1,
    column: "customer".into(),
    value: Value::Text("Updated name".into()),
}]).unwrap();
// The GPUI in-cell InputState uses the same edit transaction.`,
  },
  {
    id:'columns',group:'features',icon:'▥',en:'Columns',title:'列管理 Columns',short:'列定义与操作',
    status:'live',summary:'调整列宽、拖动列、固定左侧列，以及通过 Rust ColumnState 显示或隐藏字段。',
    tags:['Resize','Reorder','Pinned','Visibility'],
    demoContext:'拖动表头调整列顺序和宽度，也可点击 Hide Growth / Show Growth。',
    hints:['拖动列分隔线调整宽度','拖动表头移动列','Hide Growth / Show Growth 切换列可见性','Order ID 和 Customer 默认固定在左侧'],
    description:'GridModel.state.columns 维护展示列顺序、宽度、显隐与固定状态。数据 Rows 仍保持其原始存储顺序。GridDelegate 仅在列结构发生变化时重建 GPUI presentation cache，避免把屏幕列下标当成持久数据字段 ID。',
    steps:['Column 定义字段名、类型、编辑权限与默认宽度。','在 GPUI 表头拖动列以改变展示顺序。','调用 set_column_visibility 时重建列展示但不改变原始 Rows。'],
    api:[['Column::new(key, label, DataType)','创建类型化列定义'],['GridDelegate::set_column_visibility(key, bool)','通过 Rust 显示/隐藏列'],['GridModel::move_column(key, index)','修改展示列序'],['GridModel::state.columns','获取状态而不改动数据布局']],
    notes:['列工具面板和完整的宽度持久化仍在 #9 开发中。','部分列排序/移动/固定交互由 GPUI Kit DataTable 提供。'],
    code:`use gpui_grid::{GridDelegate, sample_orders};

let mut grid = GridDelegate::new(sample_orders(12_480));
grid.set_column_visibility("growth", false).unwrap();
// The row storage is unchanged, even when presentation columns move.`,
  },
  {
    id:'virtualization',group:'features',icon:'▧',en:'Virtualization',title:'虚拟滚动 Virtualization',short:'虚拟滚动',
    status:'live',summary:'50,000 行带类型的 Rust 样例数据，通过 GPUI Kit DataTable 的虚拟列表在 Canvas 中按视口绘制。',
    tags:['50,000 rows','Virtual viewport','Rust model'],
    demoContext:'向下滚动数据区：Rust 保存 50,000 行数据，但 GPUI 渲染只覆盖当前虚拟视口。',
    hints:['滚动到中间或底部观察虚拟行加载','列头仍可以排序','Shift+方向键仍使用稳定 Row ID','示例仅代表本地数据，不是远程分页'],
    description:'GPUI Kit DataTable 管理屏幕可见行和列的渲染。GridModel 保留全部本地 typed rows，GridDelegate 映射为当前排序/过滤后的可见索引。此功能页加载 50,000 行，以展示基本虚拟化；远程百万行分页与完整性能基准尚待实现。',
    steps:['使用 sample_orders(50_000) 创建 Row 数据。','TableState 将可见行列表映射至 DataTable 虚拟视口。','在滚动过程中只请求需要渲染的 GPUI cells。'],
    api:[['sample_orders(50_000)','构建本地大样例'],['GridDelegate::row_count()','当前过滤后的逻辑行数'],['DataTable::new(&table)','真正 GPUI 虚拟化渲染器']],
    notes:['50k 是功能展示量级，不构成 FPS 或性能 SLA。','100k/200 列系统性能基准在 #11。','数据一次性在浏览器 WASM 内存中构建，不是远程 RowModel。'],
    code:`use gpui_grid::{GridDelegate, sample_orders};
use gpui_kit::component::table::{DataTable, TableState};

let model = sample_orders(50_000);
let state = cx.new(|cx| {
    TableState::new(GridDelegate::new(model), window, cx)
});
let grid = DataTable::new(&state).bordered(true);
// GPUI Kit virtualizes viewport rows and columns.`,
  },
  {
    id:'grouping',group:'advanced',icon:'⊞',en:'Row Grouping',title:'行分组 Grouping',short:'行分组与聚合',
    status:'core',summary:'Rust Core 已有 group_by 和 sum/count 基础，但尚未完成 GPUI 分组行、展开折叠与汇总 UI。',
    tags:['Core API','UI in progress','#10'],
    demoContext:'尚无可操作的 GPUI 分组行演示。',
    description:'分组引擎当前能按照一个字段生成组节点并计算金额合计，但这些结果还不能作为 DataTable 可展开的 group rows 显示。需要独立行节点类型、展开状态和视口映射。',
    steps:['定义数据行与分组字段。','将分组索引输出转换为可展开的节点结构（计划）。','在 GPUI 数据表中渲染 group header / subtotal（计划）。'],
    api:[['GridModel::group_by(&indices, key, sum_col)','已经存在的纯 Rust 分组聚合 API'],['GridRowNode','计划中的 GPUI GroupRow 节点，尚未实现']],
    notes:['Core-only 并不代表浏览器 UI 已经支持。','开发任务：#10。'],
    code:`// Core-only API — no live GPUI grouping UI yet.
let model = sample_orders(12_480);
let groups = model.group_by(&model.view(), "region", "amount")?;`,
  },
  {
    id:'clipboard',group:'advanced',icon:'⧉',en:'Clipboard',title:'剪贴板 Clipboard',short:'复制与粘贴',
    status:'planned',summary:'规划：类电子表格的 TSV 范围复制、批量粘贴、原子校验与 Undo / Redo。',
    tags:['Planned','#6'],
    demoContext:'Clipboard 仍在开发，当前没有真实 GPUI WASM 示例。',
    description:'复制/粘贴会建立在稳定 CellRange、typed Edit 和编辑事务之上。必须保证非法值不会造成部分数据写入。',
    steps:['将 CellRange 转换为选中行列。','按可见列顺序编码 TSV。','粘贴时先验证全部单元格，再原子提交。'],
    api:[['ClipboardManager','规划中，尚未发布'],['GridModel::apply_edits','现有原子编辑能力，后续用于批量粘贴']],
    notes:['尚未提供操作界面。','开发任务：#6。'],
    code:`// Planned — the clipboard facade is not published yet.`,
  },
  {
    id:'server-side',group:'advanced',icon:'☁',en:'Server-side Data',title:'远程数据 Server-side',short:'远程行模型',
    status:'planned',summary:'规划：异步数据源、分页缓存、请求取消和远程行数未知时的虚拟滚动。',
    tags:['Planned','#12'],
    demoContext:'尚无真实的远程 RowModel 在线示例。',
    description:'当前 GridModel 一次性存储 Vec<Row>；远程 RowModel 需要可插拔 fetch_range 和缓存，以避免一次加载百万行。',
    steps:['定义 RowModel/DataSource 接口。','将查询排序过滤下推服务端。','缓存视口附近的数据并处理加载/失败。'],
    api:[['RowModel','设计中，尚未发布'],['AsyncDataSource','设计中，尚未发布']],
    notes:['目前 50,000 行示例是本地数据，不是服务端渲染。','开发任务：#12。'],
    code:`// Planned — AsyncDataSource and server-side RowModel are not implemented.`,
  },
  {
    id:'pivot',group:'advanced',icon:'▦',en:'Pivot & Tree Data',title:'透视与树形数据 Pivot / Tree',short:'透视与树形数据',
    status:'planned',summary:'高级功能的长期规划：Pivot、Tree Data、合并单元格与填充柄。',
    tags:['Planned','#16'],
    demoContext:'该功能正在规划，目前没有真实 GPUI 示例。',
    description:'动态列生成、嵌套数据节点、合并单元格与虚拟化需要新的布局规则；不会在功能尚未实现时发布 JavaScript 仿制示例。',
    steps:['设计并审核核心数据结构。','建立 GPUI 行列布局原型。','通过真实性能基准和跨端测试后发布。'],
    api:[['PivotEngine','规划中的 API，未发布'],['TreeRowModel','规划中的 API，未发布']],
    notes:['AG Grid Enterprise 是独立授权产品；本项目不复制商业模块。','开发任务：#16。'],
    code:`// Planned — no Pivot or TreeData public Rust API yet.`,
  },
];

export const LIVE_FEATURES = FEATURES.filter(feature => feature.status === 'live');
export const DEFAULT_FEATURE = FEATURES[0];
export const byFeatureId = id => FEATURES.find(feature => feature.id === id) ?? DEFAULT_FEATURE;
