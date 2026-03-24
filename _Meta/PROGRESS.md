# Appnut 项目进度追踪

> 最后更新：2026-03-12

---

## 一、框架层进度

### 1.1 DSL 建模系统

| 子项 | 状态 | 详情 |
|------|------|------|
| `#[model]` 宏 | ✅ 完成 | 生成 Serde、字段常量、DSL IR，已在 TwitterFlux 验证 |
| `#[facet]` 宏 | ✅ 完成 | resource + action 声明，自动生成 typed HTTP 客户端 |
| `#[dsl_enum]` 宏 | ✅ 完成 | Display/FromStr/Default/DslEnum |
| `impl_handler!` 宏 | ✅ 完成 | action 到 handler 的编译期绑定 |
| 类型系统（20+ 类型） | ✅ 完成 | Id, Name\<T\>, LocalizedText, Email, Avatar 等，31 个测试 |
| Schema 生成 | ✅ 完成 | build_schema → JSON，供 Dashboard 消费 |
| FlatBuffers 支持 | ⚠️ 部分 | 类型分类已实现，完整序列化未完成 |

### 1.2 存储层

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| KVStore trait | ✅ 完成 | — | get/set/delete/scan/batch_set/batch_delete/is_readonly |
| RedbStore | ✅ 完成 | 9 | 基于 redb，CRUD + scan + batch |
| OverlayKV | ✅ 完成 | 包含在上述 9 中 | 只读文件层 + 可写 DB 层 |
| FileLoader | ✅ 完成 | — | 从 YAML 文件加载到 OverlayKV |
| SQLStore trait | ✅ 完成 | — | query/exec |
| SqliteStore | ✅ 完成 | 9 | WAL 模式，open/open_in_memory |
| SearchEngine trait | ✅ 完成 | — | index/delete/search |
| TantivyEngine | ✅ 完成 | 7 | 多集合隔离 |
| BlobStore trait | ✅ 完成 | — | put/get/delete/exists/list/read_stream/write_stream |
| FileStore | ✅ 完成 | 9 | 路径安全，嵌套 key 支持 |

### 1.3 存储抽象层（dsl/store）

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| KvOps\<T\> | ✅ 完成 | 包含在 43 中 | get/list/count/save_new/save/patch/delete |
| SqlOps\<T\> | ✅ 完成 | 包含在 43 中 | 同上 + query |
| SearchOps\<T\> | ✅ 完成 | 包含在 43 中 | index/remove/search |
| admin_kv_router | ✅ 完成 | — | 自动 CRUD 路由生成 |
| admin_sql_router | ✅ 完成 | — | SQL 版自动路由 |
| WidgetOverride | ✅ 完成 | 包含在 43 中 | UI 控件覆盖 |
| Schema builder | ✅ 完成 | 包含在 43 中 | ModuleDef/ResourceDef/EnumDef |

### 1.4 Flux 状态引擎

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| StateStore | ✅ 完成 | 42 | BTreeMap + 订阅通知 |
| Trie 路由 | ✅ 完成 | 44 | MQTT 风格通配符 +/# |
| Router | ✅ 完成 | 19 | 请求分发 |
| Flux (App) | ✅ 完成 | 22 | get/emit/subscribe 集成 |
| StateValue | ✅ 完成 | 20 | 类型擦除 + downcast |
| I18nStore | ✅ 完成 | 15 | Trie 路径匹配翻译 |
| `#[state]` 宏 | ✅ 完成 | — | 编译期 PATH 常量 |
| `#[request]` 宏 | ✅ 完成 | — | 编译期 PATH 常量 |
| `#[flux_handlers]` 宏 | ✅ 完成 | — | handler 注册 |

### 1.5 认证与安全

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| Authenticator trait | ✅ 完成 | — | check(headers, permission) |
| AllowAll | ✅ 完成 | — | 开发用 |
| DenyAll | ✅ 完成 | — | 测试用 |
| JwtAuthenticator | ✅ 完成 | 4 | HMAC-SHA256 验证 |
| JwtService（示例用） | ✅ 完成 | 3 | token 生成 + 验证 |
| Admin API 切换到 JWT | ❌ 未完成 | — | 当前仍用 AllowAll |

### 1.6 国际化

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| I18nStore | ✅ 完成 | 15 | Trie 路径匹配 |
| LocalizedText | ✅ 完成 | 包含在类型测试中 | 多语言存储 + fallback |
| 4 语言支持 | ✅ 完成 | — | en, zh-CN, ja, es |

### 1.7 FFI 绑定

| 子项 | 状态 | 详情 |
|------|------|------|
| flux_create / flux_free | ✅ 完成 | 生命周期管理 |
| flux_server_url | ✅ 完成 | 返回嵌入服务 URL |
| flux_get | ✅ 完成 | 读取状态，返回 JSON bytes |
| flux_emit | ✅ 完成 | 发送请求 |
| flux_subscribe | ✅ 完成 | 回调 + catch_unwind |
| flux_unsubscribe | ✅ 完成 | 通过 ID 取消 |
| flux_i18n_get | ✅ 完成 | 翻译获取 |
| flux_i18n_set_locale | ✅ 完成 | 语言切换 |
| flux_bytes_free | ✅ 完成 | 内存释放 |
| flux_last_error | ✅ 完成 | 线程局部错误 |
| 错误处理（消除 unwrap） | ✅ 完成 | 所有入口 null 检查 |
| C 头文件 | ✅ 完成 | flux_ffi.h |
| cbindgen 配置 | ✅ 完成 | cbindgen.toml |
| cdylib + staticlib | ✅ 完成 | Cargo.toml crate-type |
| FFI 单元测试 | ❌ 未完成 | 当前无 Rust 端 FFI 测试 |

### 1.8 客户端 SDK

| 子项 | 状态 | 测试数 | 详情 |
|------|------|--------|------|
| Rust ResourceClient\<T\> | ✅ 完成 | 17 | list/count/get/create/update/patch/delete |
| Rust FacetClientBase | ✅ 完成 | 包含在上述中 | JSON + FlatBuffers |
| Swift FluxClient | ✅ 完成 | — | get/emit/subscribe/unsubscribe/i18n/setLocale |
| Swift FluxStore | ✅ 完成 | — | ObservableObject，订阅 #，维护 authState/timeline |
| Swift 数据模型 | ✅ 完成 | — | AuthState/Tweet/UserProfile/Message + Payload 类型 |
| Dart FluxClient | ⚠️ 部分 | — | emit/setLocale 可用，缺 get/subscribe |
| Dart 数据模型 | ✅ 完成 | — | AuthState/Tweet/UserProfile/Message |

### 1.9 Web 组件

| 子项 | 状态 | 详情 |
|------|------|------|
| 登录页 HTML | ✅ 完成 | JWT 存储，oklch 暗色主题 |
| Dashboard SPA | ✅ 完成 | Schema 驱动，自动渲染表格/表单/导航 |

### 1.10 DevOps

| 子项 | 状态 | 详情 |
|------|------|------|
| rustfmt.toml | ✅ 完成 | edition 2024, max_width 100 |
| clippy.toml | ✅ 完成 | too-many-arguments-threshold = 8 |
| GitHub Actions CI | ✅ 完成 | fmt + clippy + test |
| Dockerfile | ✅ 完成 | 多阶段构建 |
| docker-compose.yml | ✅ 完成 | 暴露 3000 端口 |
| iOS 构建脚本 | ✅ 完成 | scripts/build-ios.sh → XCFramework |
| Android 构建脚本 | ❌ 未完成 | |
| cargo doc 发布 | ❌ 未完成 | |

---

## 二、TwitterFlux 示例进度

### 2.1 后端

| 子项 | 状态 | 详情 |
|------|------|------|
| 数据模型（5 个） | ✅ 完成 | User, Tweet, Like, Follow, Message |
| Admin API（自动 CRUD） | ✅ 完成 | 5 个资源的完整 CRUD |
| App Facet API（16 个端点） | ✅ 完成 | 登录/时间线/发推/点赞/关注/搜索/上传/站内信 |
| BFF 状态 + Handler | ✅ 完成 | auth, home, tweet, user, search, inbox |
| i18n 翻译注册 | ✅ 完成 | UI 按钮/标签/站内信标题/正文 |
| 种子数据 | ✅ 完成 | 5 用户/10 推文/5 回复/17 点赞/10 关注/3 站内信 |
| Facet API 测试 | ✅ 完成 | ~36 个测试覆盖全部端点 |

### 2.2 iOS 示例 App

| 子项 | 状态 | 详情 |
|------|------|------|
| Xcode 项目骨架 | ✅ 完成 | Swift Package，依赖 FluxSDK |
| FluxSDK 集成 | ✅ 完成 | Package.swift 引用 XCFramework |
| 导航结构 | ✅ 完成 | RootView → LoginView / MainTabView(4 tabs) |
| 登录页 UI | ❌ TODO | `Views/LoginView.swift` |
| 时间线页 UI | ❌ TODO | `Views/TimelineView.swift` |
| 推文详情页 UI | ❌ TODO | `Views/TweetDetailView.swift` |
| 个人主页 UI | ❌ TODO | `Views/ProfileView.swift` |
| 发推页 UI | ❌ TODO | `Views/ComposeView.swift` |
| 搜索页 UI | ❌ TODO | `Views/SearchView.swift` |
| 站内信页 UI | ❌ TODO | `Views/InboxView.swift` |
| 设置页 UI | ❌ TODO | `Views/SettingsView.swift` |

### 2.3 Flutter 示例 App

| 子项 | 状态 | 详情 |
|------|------|------|
| Flutter 项目创建 | ✅ 完成 | example/twitter_flux/ |
| Dart FFI 客户端 | ⚠️ 部分 | emit/setLocale 可用，缺 get/subscribe |
| 数据模型 | ✅ 完成 | models.dart |
| 导航结构 | ✅ 完成 | RootView → LoginView / MainTabView(4 tabs) |
| 登录页 UI | ❌ TODO | `views/login_view.dart` |
| 时间线页 UI | ❌ TODO | `views/timeline_view.dart` |
| 搜索页 UI | ❌ TODO | `views/search_view.dart` |
| 站内信页 UI | ❌ TODO | `views/inbox_view.dart` |
| 设置页 UI | ❌ TODO | `views/settings_view.dart` |
| 推文详情页 | ❌ 未创建 | 需要新增 |
| 个人主页 | ❌ 未创建 | 需要新增 |
| 发推页 | ❌ 未创建 | 需要新增 |

---

## 三、文档进度

| 文档 | 状态 | 路径 |
|------|------|------|
| README.md | ✅ 完成 | 根目录 |
| 内部 Onboarding | ✅ 完成 | docs/internal/onboarding.md |
| AI Team 宪章 | ✅ 完成 | _Meta/CHARTER.md |
| 产品需求文档 | ✅ 完成 | _Meta/PRDs/0001-product-requirements.md |
| 项目进度追踪 | ✅ 完成 | _Meta/PROGRESS.md（本文件） |
| API 文档（cargo doc） | ❌ 未完成 | 需要补充文档注释 |
| iOS 示例构建指南 | ⚠️ 部分 | README 中有简要说明 |
| Flutter 示例构建指南 | ❌ 未完成 | |

---

## 四、测试覆盖汇总

| 模块 | 测试数 | 覆盖评估 |
|------|--------|---------|
| bff/runtime | 162 | 充分 |
| dsl/store | 43 | 充分 |
| dsl/types | 31 | 充分 |
| dsl/client | 17 | 良好 |
| dsl/macro | 12 | 基础（过程宏） |
| support/core | 13 | 良好 |
| support/kv | 9 | 良好 |
| support/sql | 9 | 良好 |
| support/search | 7 | 基础 |
| support/blob | 9 | 良好 |
| 示例 facet_handlers | ~36 | 充分 |
| bff/ffi | 0 | ❌ 缺失 |
| dsl/web | 0 | N/A（静态 HTML） |
| **合计** | **~348** | |

---

## 五、待办事项清单

### 高优先级

- [ ] Admin API 从 AllowAll 切换到 JwtAuthenticator
- [ ] Dart FluxClient 补全 get/subscribe
- [ ] iOS 示例 App 8 个页面 UI 实现（用户手写）
- [ ] Flutter 示例 App 补齐缺失页面 + UI 实现（用户手写）

### 中优先级

- [ ] FFI 层 Rust 端单元测试
- [ ] Android 构建脚本 + NDK 交叉编译
- [ ] cargo doc 文档注释补充
- [ ] FlatBuffers 完整序列化支持
- [ ] Flutter 示例构建指南文档

### 低优先级

- [ ] 更多存储后端（PostgreSQL、Redis）
- [ ] Web 前端示例（Wasm 绑定）
- [ ] Desktop 示例（Tauri）
- [ ] 发布到 crates.io
