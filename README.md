# Appnut

**Rust 跨平台应用快速开发框架**

Appnut 是一个开源的 Rust 全栈应用开发框架，核心理念是 **Rust 掌管所有状态与业务逻辑，各端（iOS / Android / Web / Desktop）只负责渲染 UI**。通过 DSL 宏驱动的领域建模、Flux 状态引擎、自动化 CRUD 和 Schema 驱动的管理后台，开发者可以用极少的代码构建完整的多平台应用。

## 目录

- [项目愿景](#项目愿景)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [核心模块详解](#核心模块详解)
  - [Flux 状态引擎 (BFF)](#flux-状态引擎-bff)
  - [DSL 领域建模](#dsl-领域建模)
  - [基础设施层 (Support)](#基础设施层-support)
- [架构总览](#架构总览)
- [示例应用：TwitterFlux](#示例应用twitterflux)
- [快速开始](#快速开始)
- [API 路由一览](#api-路由一览)
- [国际化 (i18n)](#国际化-i18n)
- [项目当前进度](#项目当前进度)
- [License](#license)

## 项目愿景

传统跨平台开发面临一个核心问题：**业务逻辑在多端重复实现**。Appnut 的解决方案是：

1. **Rust 作为唯一逻辑层**：所有状态管理、数据处理、API 调用、验证逻辑都在 Rust 中完成
2. **平台端只做渲染**：iOS (Swift)、Android (Kotlin)、Web (JS)、Desktop 通过 FFI 或 HTTP 与 Rust 层交互
3. **DSL 驱动**：通过 proc-macro 声明式定义领域模型，自动生成 CRUD、Schema、Admin API
4. **零拷贝状态共享**：基于 `Arc` 的状态引擎，避免跨层数据序列化开销

```
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│  iOS UI  │  │Android UI│  │  Web UI  │  │Desktop UI│
└────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │             │             │
     └─────────────┴──────┬──────┴─────────────┘
                          │ FFI / HTTP
                   ┌──────┴──────┐
                   │  Flux BFF   │  ← Rust 状态引擎
                   ├─────────────┤
                   │  DSL Layer  │  ← 领域模型 & API
                   ├─────────────┤
                   │   Support   │  ← KV / SQL / Search / Blob
                   └─────────────┘
```

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 语言 | Rust (edition 2024) | 核心开发语言 |
| Web 框架 | Axum 0.8 | HTTP 路由与中间件 |
| 异步运行时 | Tokio | async/await 运行时 |
| KV 存储 | redb 2 | 嵌入式键值数据库 |
| 关系数据库 | rusqlite 0.32 | SQLite（bundled） |
| 全文搜索 | Tantivy 0.22 | Rust 原生搜索引擎 |
| 二进制协议 | FlatBuffers 25 | 零拷贝序列化 |
| 认证 | jsonwebtoken 9 | JWT 签发与验证 |
| HTTP 客户端 | reqwest 0.12 | 调用外部 API |
| 序列化 | serde / serde_json | JSON 处理 |
| 可观测性 | tracing | 结构化日志 |

## 项目结构

```
appnut/
├── Cargo.toml                    # Workspace 根配置
├── Cargo.lock
│
├── crates/
│   ├── bff/                      # BFF 层 — Flux 状态引擎
│   │   ├── runtime/              #   Flux 核心（状态管理、路由、Trie、i18n）
│   │   ├── derive/               #   proc-macro（#[state], #[request], #[flux_handlers]）
│   │   └── ffi/                  #   C FFI 绑定，供 iOS/Android 调用
│   │
│   ├── dsl/                      # DSL 层 — 领域建模与 API
│   │   ├── types/                #   通用类型（Id, Name, DateTime, LocalizedText 等）
│   │   ├── macro/                #   proc-macro（#[model], #[facet], #[resource], #[action]）
│   │   ├── store/                #   存储抽象（KvOps, SqlOps, SearchOps, Schema 构建）
│   │   ├── client/               #   类型安全的 HTTP 客户端 SDK
│   │   └── web/                  #   内置 Web 组件（登录页、管理后台 Dashboard）
│   │
│   └── support/                  # 基础设施层
│       ├── core/                 #   配置、认证、错误处理、工具函数
│       ├── kv/                   #   KV 存储（redb 封装）
│       ├── sql/                  #   SQL 存储（rusqlite 封装）
│       ├── search/               #   全文搜索（Tantivy 封装）
│       └── blob/                 #   大对象存储
│
└── example/
    └── rust/                     # 示例应用 — TwitterFlux
        ├── main.rs               #   服务入口（路由组装、种子数据）
        ├── server/               #   后端（DSL 模型定义、Admin API、Facet API）
        │   ├── dsl/model/        #     领域模型（User, Tweet, Like, Follow, Message）
        │   ├── dsl/rest/         #     REST API 定义（Admin + App Facet）
        │   └── src/              #     JWT、Facet 业务处理器
        └── bff/                  #   BFF 层（状态、请求、Flux 处理器、i18n）
            ├── dsl/state/        #     状态定义（AuthState, TimelineFeed 等）
            ├── dsl/request/      #     请求定义（LoginReq, CreateTweetReq 等）
            └── src/              #     Flux handlers、i18n 翻译字符串
```

## 核心模块详解

### Flux 状态引擎 (BFF)

> 📦 `crates/bff/runtime` — crate 名 `openerp-flux`

Flux 是项目的核心，一个基于路径的跨平台状态引擎。它只有三个基本操作：

| 操作 | 说明 | 示例 |
|------|------|------|
| `get(path)` | 读取状态，Arc 零拷贝 | `app.get("auth/state")` |
| `emit(path, payload)` | 发送请求，Trie 路由到 handler | `app.emit("auth/login", creds).await` |
| `subscribe(pattern)` | 订阅状态变化，支持通配符 | `app.subscribe("timeline/#", callback)` |

**路径命名规范：**

```
auth/state              # 全局状态
app/route               # 当前路由
timeline/feed           # 时间线数据
home/devices/items/{id} # 带 ID 的条目
```

**Trie 通配符匹配（MQTT 风格）：**

| 模式 | 说明 | 匹配示例 |
|------|------|---------|
| `auth/state` | 精确匹配 | `auth/state` |
| `auth/+` | 单级通配 | `auth/state`、`auth/terms` |
| `home/#` | 多级通配 | `home/` 下所有路径 |
| `#` | 全局通配 | 所有路径 |

**使用示例：**

```rust
use flux::Flux;

let app = Flux::new();

// 注册请求处理器
app.on("app/initialize", |_, _, store| async move {
    store.set("auth/state", AuthState::unauthenticated());
    store.set("app/route", "/onboarding".to_string());
});

// 订阅状态变化
app.subscribe("#", |path, value| {
    println!("state changed: {}", path);
});

// 发送请求
app.emit("app/initialize", ()).await;
```

#### Flux Derive 宏

> 📦 `crates/bff/derive` — crate 名 `flux-derive`

通过 proc-macro 简化 BFF 开发：

- **`#[state("path")]`**：为状态类型生成 `PATH` 常量和 `StatePath` trait
- **`#[request("path")]`**：为请求类型生成路径常量和 `RequestPath` trait
- **`#[flux_handlers]`**：将 `impl` 块中的方法自动注册为 Flux 路由处理器

#### FFI 绑定

> 📦 `crates/bff/ffi` — crate 名 `flux-ffi`

提供 C ABI 导出，供 iOS (Swift) 和 Android (Kotlin) 通过 FFI 调用 Flux 状态引擎。

### DSL 领域建模

#### 类型系统

> 📦 `crates/dsl/types` — crate 名 `openerp-types`

提供丰富的领域类型，不只是简单的 `String`：

| 类型 | 用途 |
|------|------|
| `Id` | UUID 主键 |
| `Name<T>` | 类型安全的资源引用（如 `twitter/users/alice`） |
| `Email`, `Phone`, `Url` | 语义化字符串类型 |
| `Password`, `PasswordHash`, `Secret` | 安全敏感类型 |
| `DateTime`, `Date` | 时间类型 |
| `Text`, `Markdown`, `Code` | 文本类型 |
| `Avatar`, `ImageUrl` | 媒体类型 |
| `LocalizedText` | 多语言文本，按 locale 自动选择 |
| `Color`, `SemVer` | 其他常用类型 |

核心 trait：

- **`DslModel`**：标记领域模型，提供模块名、资源名、验证等元信息
- **`NameTarget`**：验证 `Name<T>` 引用是否合法
- **`NameTemplate`**：定义资源的名称前缀和模板

#### DSL 宏

> 📦 `crates/dsl/macro` — crate 名 `openerp-macro`

通过 proc-macro 声明式定义模型和 API：

- **`#[model(module, name)]`**：生成 serde、DslModel、IR 定义、CRUD 代码
- **`#[facet(name, module)]`**：定义面向 App 的 API 接口（Facet 模块）
- **`#[resource(path, pk)]`**：定义资源投影（只读视图）
- **`#[action(method, path)]`**：定义操作签名

```rust
#[model(module = "twitter", name = "user")]
pub struct User {
    pub id: Id,
    pub username: String,
    pub password_hash: Option<PasswordHash>,
    pub bio: Option<String>,
    pub avatar: Option<Avatar>,
    pub follower_count: i64,
    pub following_count: i64,
    pub tweet_count: i64,
    // ...
}
```

#### 存储抽象

> 📦 `crates/dsl/store` — crate 名 `openerp-store`

统一的存储操作接口：

- **`KvOps<T>`**：KV 存储 CRUD，自动校验 `Name<T>` 引用
- **`SqlOps<T>`**：SQL 存储 CRUD
- **`SearchOps<T>`**：全文搜索索引与查询
- **`build_schema()`**：从 DSL 模型 IR 构建 JSON Schema
- **`admin_kv_router()`**：自动生成 Admin CRUD REST API

#### HTTP 客户端

> 📦 `crates/dsl/client` — crate 名 `openerp-client`

类型安全的 HTTP 客户端 SDK：

- **`ResourceClient<T>`**：CRUD 客户端（list / get / create / update / patch / delete）
- **`FacetClientBase`**：Facet API 客户端，支持 JSON 和 FlatBuffers
- **`TokenSource`**：认证方式（NoAuth / StaticToken / PasswordLogin 自动刷新）

#### Web 组件

> 📦 `crates/dsl/web` — crate 名 `openerp-web`

内置的 Web 管理界面：

- **登录页**（`login.html`）：标准登录表单，POST `/auth/login`，JWT 存入 localStorage
- **管理后台**（`dashboard.html`）：Schema 驱动的 SPA，根据 `/meta/schema` 自动生成 CRUD 表格和表单

Dashboard 特点：
- 暗色主题（oklch 色彩系统）
- Phosphor Icons 图标库
- 完全由 Schema 驱动，无需手写管理界面
- 支持列表、新增、编辑、删除操作

### 基础设施层 (Support)

#### 核心服务

> 📦 `crates/support/core` — crate 名 `openerp-core`

| 模块 | 功能 |
|------|------|
| `config.rs` | 服务配置（从 CLI 参数解析） |
| `auth.rs` | 认证抽象（`Authenticator` trait、`AllowAll`、`DenyAll`） |
| `error.rs` | 统一错误类型（`ServiceError`：NotFound、Conflict、Validation 等） |
| `types.rs` | 工具函数（`new_id()`、`now_rfc3339()`、`merge_patch()` RFC 7386） |

配置参数：

```
--data-dir=<path>     # 数据根目录
--db=<path>           # redb 数据库路径
--sqlite=<path>       # SQLite 数据库路径
--search-dir=<path>   # Tantivy 索引目录
--blob-dir=<path>     # Blob 存储目录
--listen=<addr>       # 监听地址（默认 0.0.0.0:8080）
```

#### KV 存储

> 📦 `crates/support/kv` — crate 名 `openerp-kv`

基于 [redb](https://github.com/cberner/redb) 的嵌入式键值存储：

- `KVStore` trait：set / get / delete / scan / count
- `RedbStore`：redb 实现
- 带 Criterion 基准测试

#### SQL 存储

> 📦 `crates/support/sql` — crate 名 `openerp-sql`

基于 rusqlite 的 SQLite 封装：

- `SqlStore` trait：insert / query / range / update / delete
- 带 Criterion 基准测试

#### 全文搜索

> 📦 `crates/support/search` — crate 名 `openerp-search`

基于 [Tantivy](https://github.com/quickwit-oss/tantivy) 的全文搜索：

- `SearchStore` trait：index / search / delete
- 带 Criterion 基准测试

#### Blob 存储

> 📦 `crates/support/blob` — crate 名 `openerp-blob`

大对象（文件）存储：

- `BlobStore` trait：put / get / list / delete
- 带 Criterion 基准测试

## 架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                        示例应用 (example/rust)                       │
├──────────────────────┬──────────────────────────────────────────────┤
│  Server 层           │  BFF 层                                      │
│  ├─ DSL 模型定义      │  ├─ 状态定义（#[state]）                      │
│  ├─ Admin REST API   │  ├─ 请求定义（#[request]）                    │
│  ├─ App Facet API    │  ├─ Flux handlers（业务逻辑）                  │
│  └─ JWT 认证         │  └─ i18n 翻译字符串                           │
└──────────────────────┴──────────────────────────────────────────────┘
        │                         │
        ▼                         ▼
┌───────────────┐  ┌──────────────────────┐  ┌────────────────────┐
│ crates/dsl    │  │ crates/bff           │  │ crates/support     │
│ ├─ types      │  │ ├─ runtime (Flux)    │  │ ├─ core            │
│ ├─ macro      │  │ ├─ derive            │  │ ├─ kv (redb)       │
│ ├─ store      │  │ └─ ffi               │  │ ├─ sql (sqlite)    │
│ ├─ client     │  │                      │  │ ├─ search(tantivy) │
│ └─ web        │  │                      │  │ └─ blob            │
└───────────────┘  └──────────────────────┘  └────────────────────┘
```

**请求处理流程：**

```
客户端 UI
    │
    │ ① emit("auth/login", {username, password})
    ▼
Flux 状态引擎
    │
    │ ② Trie 路由匹配 → 找到 login handler
    ▼
BFF Handler
    │
    │ ③ 调用 Facet API (HTTP Client)
    ▼
Server Facet API
    │
    │ ④ JWT 校验 → 业务处理 → KvOps CRUD
    ▼
KV / SQL / Search 存储
    │
    │ ⑤ 返回结果
    ▼
BFF Handler
    │
    │ ⑥ store.set("auth/state", AuthState { ... })
    ▼
StateStore
    │
    │ ⑦ Trie 匹配订阅者 → 通知 UI
    ▼
客户端 UI 更新
```

## 示例应用：TwitterFlux

`example/rust/` 包含一个完整的类 Twitter 应用，展示了框架的全部能力：

### 领域模型

| 模型 | 说明 | 关键字段 |
|------|------|---------|
| **User** | 用户 | username, password_hash, bio, avatar, follower_count |
| **Tweet** | 推文 | author (`Name<User>`), content, like_count, reply_to |
| **Like** | 点赞 | user (`Name<User>`), tweet (`Name<Tweet>`) |
| **Follow** | 关注 | follower (`Name<User>`), followee (`Name<User>`) |
| **Message** | 站内信 | title (`LocalizedText`), body (`LocalizedText`), read |

### BFF 状态

| 状态 | 路径 | 说明 |
|------|------|------|
| AuthState | `auth/state` | 登录状态、当前用户 |
| AppRoute | `app/route` | 当前路由 |
| TimelineFeed | `timeline/feed` | 时间线推文列表 |
| ComposeState | `compose/state` | 发推草稿 |
| SearchState | `search/state` | 搜索结果 |
| InboxState | `inbox/state` | 站内信箱 |
| ProfilePage | `profile/page` | 用户主页 |
| TweetDetail | `tweet/detail` | 推文详情 |
| SettingsState | `settings/state` | 用户设置 |

### 种子数据

启动后自动填充测试数据：
- 5 个用户（alice, bob, carol, dave, eve）
- 10 条推文 + 5 条回复
- 17 个点赞
- 10 个关注关系
- 3 条站内信（含多语言内容）
- 所有用户密码均为 `password`

## 快速开始

### 环境要求

- **Rust**：需要支持 edition 2024 的 nightly 版本（或将 `Cargo.toml` 中的 `edition = "2024"` 改为 `"2021"`）
- 无需额外安装数据库，所有存储均为嵌入式

### 运行示例应用

```bash
# 克隆项目
git clone https://github.com/your-org/appnut.git
cd appnut

# 运行 TwitterFlux 示例
cargo run -p flux-golden

# 或者使用 Bazel（如果已配置）
# bazel run //e2e/flux/rust:twitterd
```

启动后：

- 登录页：http://localhost:3000/
- 管理后台：http://localhost:3000/dashboard
- 健康检查：http://localhost:3000/health
- Schema：http://localhost:3000/meta/schema
- 登录凭据：用户名 `root`（或任意种子用户），密码任意

### Docker 运行

```bash
docker compose up --build
```

### 构建 iOS 库

```bash
# 编译 Rust → iOS 静态库 + XCFramework
./scripts/build-ios.sh

# 然后在 Xcode 中打开 example/ios/TwitterFlux/
```

### 运行测试

```bash
# 单元测试
cargo test

# 基准测试
cargo bench
```

## API 路由一览

### 公共路由

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 登录页 |
| GET | `/dashboard` | 管理后台 SPA |
| GET | `/meta/schema` | 应用 Schema（JSON） |
| GET | `/health` | 健康检查 |
| POST | `/auth/login` | 登录（返回 JWT） |

### Admin API（`/admin/twitter/`）

由 `admin_kv_router()` 从 DSL 模型自动生成：

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/{resources}` | 资源列表 |
| GET | `/{resources}/{id}` | 获取单条 |
| POST | `/{resources}` | 创建资源 |
| PUT | `/{resources}/{id}` | 全量更新 |
| PATCH | `/{resources}/{id}` | 部分更新（RFC 7386 Merge Patch） |
| DELETE | `/{resources}/{id}` | 删除资源 |
| GET | `/{resources}/@count` | 资源计数 |

### App Facet API（`/app/twitter/`，需 JWT）

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/auth/login` | 应用登录 |
| POST | `/me` | 获取当前用户 |
| POST | `/timeline` | 时间线 |
| POST | `/tweets` | 发布推文 |
| POST | `/tweets/{id}/detail` | 推文详情 |
| POST | `/tweets/{id}/like` | 点赞 |
| POST | `/users/{id}/follow` | 关注用户 |
| POST | `/search` | 搜索 |
| POST | `/inbox` | 站内信列表 |
| POST | `/messages/{id}/read` | 标记已读 |

## 国际化 (i18n)

框架内置 i18n 支持，基于 Trie 的路径匹配翻译系统：

**支持语言**：`en`、`zh-CN`、`ja`、`es`

**使用方式：**

```rust
let i18n = I18nStore::new();

// 注册翻译
i18n.handle("ui/button/sign_in", |locale, _| match locale {
    "zh-CN" => "登录".into(),
    "ja"    => "ログイン".into(),
    "es"    => "Iniciar sesión".into(),
    _       => "Sign In".into(),
});

// 获取翻译（支持查询参数）
let text = i18n.get("ui/button/sign_in", "zh-CN"); // → "登录"
```

`LocalizedText` 类型用于数据库中存储多语言内容（如站内信），按客户端 locale 自动选择最佳语言。

## 项目当前进度

### 已完成

- [x] Cargo workspace 搭建（15 个 crates）
- [x] Flux 状态引擎核心（get / emit / subscribe + Trie 路由）
- [x] DSL proc-macro（#[model]、#[facet]、#[state]、#[request]、#[flux_handlers]）
- [x] 丰富的类型系统（Id、Name\<T\>、LocalizedText 等 20+ 类型）
- [x] KV 存储 + 自动 Admin CRUD Router
- [x] SQL 存储、全文搜索、Blob 存储基础封装
- [x] Schema 驱动的管理后台（登录页 + Dashboard）
- [x] 类型安全的 HTTP 客户端 SDK
- [x] i18n（4 语言，Trie 模式匹配）
- [x] FFI C 绑定（subscribe 回调、错误处理、C 头文件）
- [x] iOS XCFramework 构建脚本 + Swift 封装层（FluxClient / FluxStore）
- [x] iOS 示例 App 骨架（SwiftUI，页面待实现）
- [x] Flutter 示例 App 骨架（Dart FFI 客户端，页面待实现）
- [x] 完整的 TwitterFlux 后端示例应用
- [x] Criterion 基准测试（kv、sql、search、blob）
- [x] 全层单元测试（support/kv、sql、search、blob + 框架层）
- [x] JWT Authenticator 中间件
- [x] GitHub Actions CI（fmt + clippy + test）
- [x] Docker 容器化

### 待完成

- [ ] iOS 示例 App 页面实现（SwiftUI）
- [ ] Flutter 示例 App 页面实现
- [ ] Web 前端示例（Wasm 或 JS 绑定）
- [ ] Desktop 示例（Tauri 集成）
- [ ] 更多存储后端（PostgreSQL、Redis 等）
- [ ] API 文档（Rust doc 发布）
- [ ] 发布到 crates.io

## License

MIT
