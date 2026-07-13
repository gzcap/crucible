# Roc：基于 Tauri 2 的 Obsidian 风格笔记应用 — 实现方案

## Context（背景）

当前 `/Users/zmh/Documents/code/roc/roc` 是 Tauri 2 默认 vanilla-JS 模板（仅 `greet` 命令 + 欢迎页），无任何业务逻辑。目标是把它改造为一个 Mac 上可运行的 Obsidian 风格本地知识库应用：选定本地文件夹作为 Vault、监听文件变更、Markdown 双模式编辑、`[[wikilink]]`/`#tag` 解析、反向链接索引、全文搜索、链接图谱可视化。

**本轮交付边界**（已与用户确认）：
- ✅ 核心层全部可用：Vault 多库切换、文件监听+重命名自动修复链接、Markdown 解析、双模式编辑器（源码/阅读预览）、反向链接索引、全文搜索（tantivy）、图谱可视化。
- ⏸ 延迟实现（仅留架构 hook）：Canvas、插件机制、CSS 片段、模板、Dataview。
- 语言：**TypeScript**（前端）+ **Rust**（后端）。
- 索引存储：**内存 DashMap**（链接图，启动重建）+ **tantivy 落盘**（全文搜索）。无 sqlite。

---

## 技术栈与版本（已验证 2026-07）

**前端（npm）**：vue ^3.5、vue-router ^5.0.7（`createWebHashHistory`）、pinia ^3.0.4；`@tauri-apps/api` ^2.11.1、`@tauri-apps/cli` ^2.11.4、`@tauri-apps/plugin-fs` ^2.5.1、`@tauri-apps/plugin-dialog` ^2.7.1、`@tauri-apps/plugin-os` ^2.3.2、`@tauri-apps/plugin-opener` ^2；`@vitejs/plugin-vue` ^5.2、vite ^5.4、typescript ^5.6、vue-tsc ^2.1。
**CodeMirror 6**：codemirror、@codemirror/{state@6.7,view@6.43,language@6.12,lang-markdown@6.5,commands@6.10,search@6.7,autocomplete@6.20,theme-one-dark@6.1,language-data@6}。
**预览/渲染**：markdown-it ^14.3、@vscode/markdown-it-katex ^1.1.2、katex ^0.17、markdown-it-task-lists ^2、highlight.js ^11.11。
**图谱**：force-graph ^1.51.4（vanilla，挂 div，`shallowRef` 持有）。

**后端（Cargo）**：tauri 2、tauri-plugin-{opener,fs,dialog,os} 2；notify 8.2 + notify-debouncer-full 0.7（**full** 非 mini，用于 rename 配对）；tantivy 0.26（用 `TantivyDocument`、`NumericOptions`、`DateTimePrecision`）；walkdir 2.5、regex 1.13、parking_lot 0.12、dashmap 6.2、time 0.3、anyhow 1、thiserror 2。

**关键 API 约定**：
- Tauri 2 emit：`use tauri::Emitter; app.emit("event", payload)`，payload 需 `Serialize+Clone`。
- `State<T>` 要求 `T: Send+Sync+'static`；DashMap 直接可用。
- vault 任意路径需运行时放行：`use tauri_plugin_fs::FsExt; app.fs_scope().allow_directory(&vault, true)`。
- CM6 `EditorView` 必须 `shallowRef`（Vue reactive Proxy 破坏 WeakMap）。
- Vue Router 必须用 `createWebHashHistory`（Tauri 无服务器，history 模式刷新 404）。

---

## 目录结构

### 根
```
index.html              # Vite 入口（从 src/ 迁到根），<script src="/src/main.ts">
package.json            # dev/build/tauri scripts
vite.config.ts          # vue 插件 + @ 别名 + strictPort:5173 + 忽略 src-tauri
tsconfig.json / tsconfig.node.json
README.md               # 阶段 9 重写
```

### 前端 `src/`
```
main.ts                 # createApp(App).use(pinia).use(router).mount('#app')
App.vue                 # 布局 + 全局 Cmd+K 监听
router/index.ts         # createWebHashHistory
stores/{vault,notes,editor,search,graph}.ts
components/
  layout/{AppLayout,Sidebar,StatusBar}.vue
  vault/{VaultPicker,VaultSwitcher}.vue
  explorer/{FileTree,FileTreeNode}.vue
  editor/{EditorPane,PreviewPane,EditorToolbar}.vue
  backlinks/BacklinksPanel.vue
  search/CommandPalette.vue
  graph/GraphView.vue
lib/
  tauri.ts              # 类型化 invoke/listen 封装
  types.ts              # 与 Rust 对齐的 TS 类型
  markdown/{markdown,highlight}.ts
  codemirror/{extensions,wikilinks,tags,autocomplete}.ts
  forcegraph/buildGraph.ts
styles/{main,themes,codemirror,graph}.css
```

### 后端 `src-tauri/src/`
```
main.rs                 # 不变
lib.rs                  # run()：注册插件、manage(AppState)、命令、启动逻辑
error.rs                # AppError(thiserror+Serialize) + Result 别名
state.rs                # AppState（State<> 持有）
commands.rs             # 所有 #[tauri::command]（分组 re-export）
vault.rs                # VaultRegistry(vaults.json) + open/switch/close/remove + fs_scope 放行
watcher.rs              # notify-debouncer-full + 事件线程 + emit + rename 配对
parser.rs               # 正则提取 wikilink/tag/embed/title
index/{mod,graph,search}.rs   # LinkIndex(DashMap) + SearchIndex(tantivy) 聚合
```

---

## 分阶段实施

### 阶段 0：脚手架迁移（Vite + Vue3 + TS）
- 新建 `package.json`、`vite.config.ts`、`tsconfig*.json`、根 `index.html`、`src/main.ts`、`src/App.vue`（占位）、`src/vite-env.d.ts`、`src/styles/main.css`。
- 删除 `src/main.js`、`src/index.html`、`src/styles.css`。
- 改 `tauri.conf.json`：`frontendDist:"../dist"`、`devUrl:"http://localhost:5173"`、`beforeDevCommand:"pnpm dev"`、`beforeBuildCommand:"pnpm build"`、`withGlobalTauri:false`、窗口 1280×840。
- 改 `Cargo.toml`：加全部新依赖（见上）。
- 改 `lib.rs`：注册 fs/dialog/os 插件。
- 改 `capabilities/default.json`：见第 5 节。
- **验证**：`pnpm install && pnpm tauri dev` 打开 1280×840 窗口显示 Vue 占位页。

### 阶段 1：Vault 管理 + 文件树
- Rust：`error.rs`、`state.rs`（AppState 骨架）、`vault.rs`（`VaultRegistry`/`VaultInfo`，`vaults.json` 存 `app_data_dir/roc/`，open/switch 时 `fs_scope().allow_directory`）、`commands.rs`、`lib.rs`（`.manage(AppState)`）。
- 前端：`stores/{vault,notes}.ts`、`router/index.ts`、`App.vue`、`layout/{AppLayout,Sidebar,StatusBar}.vue`、`vault/{VaultPicker,VaultSwitcher}.vue`、`explorer/{FileTree,FileTreeNode}.vue`、`lib/{tauri,types}.ts`。文件树用 `@tauri-apps/plugin-fs` 的 `readDir` 递归建树。
- **验证**：选测试 vault → 树正确展示；重启自动恢复；多库切换。

### 阶段 2：文件监听 + 事件
- Rust：`watcher.rs`（`new_debouncer(300ms, None, callback)`，递归 watch，事件分类后 emit `roc://file-changed`）、`state.rs`（加 `watcher: Mutex<Option<Debouncer<...>>>`）、`vault.rs`（open/switch 启动、close/switch drop）。
- 前端：`stores/notes.ts` 加 `listenForFileChanges()`。
- 事件 payload：`FileChangeEvent{kind:"create"|"delete"|"rename"|"modify", path, new_path?}`，仅 `.md`。
- **验证**：Finder 增删重命名 `.md` → 树 ~300ms 同步。

### 阶段 3：Markdown 解析 + 链接索引
- Rust：`parser.rs`（`extract_links/tags/embeds/title`）、`index/{mod,graph}.rs`（`LinkIndex`：notes/forward/backrefs/name_index 四个 DashMap；`rebuild_from_vault`/`upsert_note`/`remove_note`/`on_rename`/`resolve_wikilink`/`export_graph`）、`watcher.rs`（事件触发 `upsert/remove/on_rename`）、`commands.rs`（`get_backlinks`/`get_outlinks`/`get_unresolved_links`/`get_all_tags`/`get_note_meta`/`list_all_notes`/`resolve_wikilink`）。
- **on_rename 修复**：解析旧 stem → 遍历 forward 找引用源 → 正则替换 `[[old…]]→[[new…]]`（保留 heading/alias）→ 写回 → 重解析 → emit modify 事件。
- **验证**：A 含 `[[B|别名]] #foo` → `get_backlinks(B)` 含 A；重命名 B→C → A 内容变 `[[C|别名]]`。

### 阶段 4：编辑器源码模式（CM6）+ wikilink 装饰
- 前端：`editor/{EditorPane,EditorToolbar}.vue`、`lib/codemirror/{extensions,wikilinks,tags,autocomplete}.ts`、`stores/editor.ts`（`editorView: shallowRef`）、`styles/codemirror.css`。笔记读写走 `@tauri-apps/plugin-fs` 的 `readTextFile`/`writeTextFile`。
- CM6 扩展：`history()`+`markdown()`+keymaps+`autocompletion`+`oneDark`+`lineWrapping`+wikilinks（ViewPlugin+`Decoration.widget` chip+click 委托）+tags+updateListener（同步 store、置 dirty）。
- `[[` 触发 `list_all_notes` 补全；`#` 触发 `get_all_tags` 补全。Ctrl+S 保存。
- **验证**：chip 显示+点击跳转；补全工作；保存写盘清 dirty。

### 阶段 5：阅读预览（markdown-it）
- 前端：`editor/PreviewPane.vue`、`lib/markdown/{markdown,highlight}.ts`、`EditorToolbar`（切换按钮）。
- `markdown-it({html:false,linkify:true,highlight:hljsFn})` + katex + task-lists；`md.inline.ruler.before('link','wikilink',fn)` 渲染 `<a class="wikilink" data-target>`；`after('wikilink','hashtag',fn)` 渲染 `<a class="tag" data-tag>`。容器 click 委托导航。
- **验证**：渲染 HTML；wikilink/tag 可点击；代码高亮；katex；任务列表。

### 阶段 6：反向链接面板
- 前端：`backlinks/BacklinksPanel.vue`（集成进 AppLayout 右侧可折叠）、`stores/notes.ts`（currentPath 变化调 `get_backlinks`）。
- Rust 返回 `Backlink{source, source_title, snippet}`。
- **验证**：打开 B → 列出 A（含片段）；点击 A 打开。

### 阶段 7：搜索面板 + tantivy
- Rust：`index/search.rs`（schema：`path STRING STORED`、`title TEXT STORED`、`body TEXT`、`tags TEXT STORED`、`mtime i64`；`SearchIndex` 持 `Index`/`Schema`/`Mutex<IndexWriter>`/`RwLock<IndexReader>`；目录 `<app_data_dir>/roc/indexes/<vault_id>`；`build_from_vault`/`upsert_note`(delete_term+add+commit)/`remove_note`/`search(query,mode)`）、`commands.rs`（`search`/`reindex_all`）。
- 前端：`search/CommandPalette.vue`、`stores/search.ts`、`App.vue`（Cmd/Ctrl+K）。
- `mode ∈ {"fulltext","tag","link"}`；commit 按事件批量合并，不按按键。
- **验证**：Cmd+K 搜索带高亮片段；标签/链接模式工作；点击结果导航。

### 阶段 8：图谱视图
- Rust：`commands.rs`（`get_graph()->GraphData`，从 LinkIndex 一次性导出）。
- 前端：`graph/GraphView.vue`（`shallowRef` force-graph 实例，`onNodeClick` 导航）、`lib/forcegraph/buildGraph.ts`（过滤裁剪）、`stores/graph.ts`、router 加 `/graph`。
- 容器显式宽高 + `ResizeObserver` 调 `g.width()/height()`。
- **验证**：节点+边显示；节点大小=链接数；点击导航；标签/搜索过滤。

### 阶段 9：README + 打磨
- `README.md`：架构说明、技术栈、`pnpm install`/`pnpm tauri dev`/`pnpm tauri build`、功能清单（已实现 vs 延迟）、目录结构。
- 主题（light/dark）、状态栏、空状态、错误 toast。
- **验证**：`pnpm tauri build` 产出 `.app`；README 步骤可复现。

---

## Rust 后端设计要点

### AppState（`state.rs`）
```rust
pub struct AppState {
    pub current_vault: parking_lot::RwLock<Option<VaultInfo>>,
    pub vault_registry: parking_lot::RwLock<VaultRegistry>,
    pub link_index: index::graph::LinkIndex,     // DashMap 自带同步
    pub search_index: index::search::SearchIndex, // writer 在 Mutex 内
    pub watcher: parking_lot::Mutex<Option<Debouncer<FsEventWatcher, FileIdMap>>>,
    pub app_data_dir: parking_lot::RwLock<PathBuf>,
}
```

### LinkIndex（`index/graph.rs`）
```rust
pub struct LinkIndex {
    pub notes: DashMap<String, NoteMeta>,        // path -> meta
    pub forward: DashMap<String, Vec<LinkRef>>,  // path -> 出链
    pub backrefs: DashMap<String, Vec<String>>,  // target path -> 源 path 列表
    pub name_index: DashMap<String, Vec<String>>,// stem -> 候选 path（resolve 用）
}
```

### 命令清单（`commands.rs`）
`open_vault`、`add_vault_by_path`、`list_vaults`、`switch_vault`、`close_vault`、`remove_vault`、`get_current_vault`、`get_watcher_status`、`list_all_notes`、`get_note_meta`、`get_backlinks`、`get_outlinks`、`get_unresolved_links`、`get_all_tags`、`resolve_wikilink`、`rename_note`、`delete_note`、`search`、`reindex_all`、`get_graph`。
（笔记内容读写/文件树枚举走前端 fs plugin；`rename_note`/`delete_note` 走 Rust 以保证索引与链接修复原子性。）

### 事件
| 事件 | payload | 时机 |
|---|---|---|
| `roc://file-changed` | `{kind,path,new_path?}` | watcher 检测 `.md` 变更 |
| `roc://index-updated` | `{note_path}` | 增量索引完成 |
| `roc://watcher-error` | `{message}` | watcher 出错 |
| `roc://vault-opened` | `VaultInfo` | open/switch 成功 |
| `roc://vault-closed` | `{}` | close_vault |

---

## 前端架构要点

- **Router**：`/`、`/note/:path(.*)*`、`/graph`（均渲染 `AppLayout`，内部 tab 切换）。
- **Pinia stores**：vault（列表/当前）、notes（树/当前笔记/tabs/dirty/backlinks/事件监听）、editor（mode + `shallowRef<EditorView>`）、search（开关/查询/模式/结果）、graph（data/过滤器）。
- **组件树**：App → AppLayout → [Sidebar(VaultSwitcher+FileTree) | 主区(EditorPane⇄PreviewPane / GraphView) | BacklinksPanel] + StatusBar + CommandPalette(Cmd+K)。

---

## Capabilities（`src-tauri/capabilities/default.json`）
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default", "opener:default", "dialog:default", "os:default",
    "fs:default",
    "fs:allow-write-file", "fs:allow-mkdir",
    "fs:allow-remove", "fs:allow-rename-file", "fs:allow-copy-file"
  ]
}
```
vault 任意路径由运行时 `app.fs_scope().allow_directory(&vault, true)` 放行。

---

## 验证计划

**启动**：`pnpm install && pnpm tauri dev` → 1280×840 窗口、Vue 页、热更新。

**测试 vault** `~/roc-test-vault/`：
```
A.md      : # A\n链接 [[B|别名]] 与悬空 [[Z]]\n#foo #bar
B.md      : # B\n被 A 引用\n#foo
sub/C.md  : # C\n[[A]] 嵌入 ![[B]]
```

各阶段验证要点见上各阶段“验证”。关键端到端：建 vault → 树展示 → 编辑 A 看到 chip → 重命名 B→C 后 A 自动改 `[[C|别名]]` → 反向链接面板更新 → Cmd+K 搜“引用”命中 → 图谱显示 4 节点+边 → 点节点跳转。`pnpm tauri build` 产出 `.app`。

---

## 风险与注意

1. **macOS FSEvents rename 拆分**：notify 可能将重命名拆成 Remove+Create。watcher 线程维护 500ms 待配对缓存，超时回退为独立 remove/create；空 paths 目录事件触发 rescan。
2. **CM6 shallowRef**：`EditorView` 与扩展数组都不可被 reactive 包装，否则装饰/补全失效。
3. **tantivy 0.26 API**：`TantivyDocument`、`NumericOptions`、`DateTimePrecision`；编辑=delete_term+add+commit+reader.reload()；commit 按事件批量，勿按按键。
4. **fs scope 动态放行**：open/switch 后必须 `fs_scope().allow_directory`，否则前端 fs plugin 被拒。
5. **watcher Debouncer 存活**：存入 `AppState.watcher` 的 `Mutex<Option<Debouncer>>`，close/switch 时 `take()` drop。
6. **自触发循环**：前端 `writeTextFile` 触发 watcher modify → 重解析。`upsert_note` 须对相同内容幂等；watcher 可对 modify 做 mtime+size 去重。
7. **force-graph 容器尺寸**：canvas 需显式宽高 + ResizeObserver，否则 0×0。
8. **markdown-it 安全**：`html:false`，`v-html` 仅渲染本地受信内容。

---

## 延迟功能（本轮仅留 hook，不实现）
- Canvas（画布）：`router` 预留 `/canvas/:id`，后端暂不提供 `.canvas` 文件解析。
- 插件机制：预留 `plugins/` 目录与 `PluginManifest` 类型占位，无加载器。
- CSS 片段：`styles/` 下预留 `snippets/` 目录与设置项占位。
- 模板：预留 `templates/` 目录与 `apply_template` 命令签名占位。
- Dataview：预留查询语言类型，不实现解析器。
