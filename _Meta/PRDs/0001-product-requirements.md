# PRD-0001: Appnut 产品需求总览

## 状态

已批准

## 背景

Appnut 是一个基于 Rust 的开源全栈 App 开发框架。目标是让开发者通过 DSL 声明式定义数据模型，自动生成后端 API、管理后台和多端 SDK，开发者只需编写 UI 代码即可交付完整应用。

当前以 TwitterFlux（类 Twitter 社交应用）作为示例，验证框架的完整能力。

## 目标用户

- 独立开发者 / 小团队，想快速做一个有后端的 App
- 有 iOS / Flutter / Web 前端经验，不想自己写后端
- 希望零运维（嵌入式存储，无需部署数据库）

---

## 第一部分：框架能力需求

### F-01 DSL 建模系统

用声明式宏定义数据模型，自动推导出存储、API、Schema、客户端 SDK。


| 编号      | 需求              | 说明                                                              |
| ------- | --------------- | --------------------------------------------------------------- |
| F-01-01 | `#[model]` 宏    | 定义数据模型，自动生成 Serialize/Deserialize（camelCase）、字段常量、DSL IR        |
| F-01-02 | `#[facet]` 宏    | 定义业务接口，`#[resource]` 声明资源，`#[action]` 声明动作                      |
| F-01-03 | `#[dsl_enum]` 宏 | 定义枚举类型，自动生成 Display/FromStr/Default                             |
| F-01-04 | 类型系统            | 提供 20+ 语义类型：Id, NameT, LocalizedText, Email, Avatar, DateTime 等 |
| F-01-05 | Schema 生成       | 从 DSL 模型自动生成 JSON Schema，供前端和管理后台消费                             |


### F-02 存储层

嵌入式存储，零外部依赖，支持四种存储引擎。


| 编号      | 需求      | 说明                                             |
| ------- | ------- | ---------------------------------------------- |
| F-02-01 | KV 存储   | 基于 redb，支持 CRUD、前缀扫描、批量操作、OverlayKV（只读层 + 可写层） |
| F-02-02 | SQL 存储  | 基于 rusqlite，WAL 模式，支持参数化查询                     |
| F-02-03 | 全文搜索    | 基于 Tantivy，支持多集合隔离、索引/搜索/删除                    |
| F-02-04 | Blob 存储 | 基于本地文件系统，支持流式读写、前缀列表、路径安全                      |


### F-03 自动 Admin API

从 DSL 模型自动生成 RESTful 管理接口。


| 编号      | 需求       | 说明                                                  |
| ------- | -------- | --------------------------------------------------- |
| F-03-01 | CRUD 路由  | GET（列表/单条/计数）、POST（创建）、PUT（全量更新）、PATCH（部分更新）、DELETE |
| F-03-02 | 分页       | 支持 offset + limit 分页                                |
| F-03-03 | 乐观锁      | PATCH 时通过 updatedAt 检测冲突（409）                       |
| F-03-04 | 管理后台 SPA | Schema 驱动的单页应用，自动渲染数据表格、表单、导航                       |
| F-03-05 | 登录页      | 支持 JWT 登录，token 存储在 localStorage                    |


### F-04 Flux 状态引擎（BFF）

跨端状态管理引擎，三原语架构。


| 编号      | 需求                    | 说明                                               |
| ------- | --------------------- | ------------------------------------------------ |
| F-04-01 | `get(path)`           | 读取指定路径的状态，返回类型擦除的值                               |
| F-04-02 | `emit(path, payload)` | 发送请求，经 Trie 路由分发到 handler                        |
| F-04-03 | `subscribe(pattern)`  | 订阅状态变化，支持 MQTT 风格通配符（`+` 单级，`#` 多级）              |
| F-04-04 | Trie 路由               | 精确匹配 + 通配符匹配，O(路径深度) 查找                          |
| F-04-05 | 派生宏                   | `#[state]`、`#[request]`、`#[flux_handlers]` 编译期注册 |


### F-05 认证与安全


| 编号      | 需求                  | 说明                         |
| ------- | ------------------- | -------------------------- |
| F-05-01 | Authenticator trait | 可插拔认证接口                    |
| F-05-02 | JWT 认证              | HMAC-SHA256，支持 token 生成和验证 |
| F-05-03 | AllowAll / DenyAll  | 开发/测试用快捷认证器                |


### F-06 国际化（i18n）


| 编号      | 需求               | 说明                               |
| ------- | ---------------- | -------------------------------- |
| F-06-01 | Trie 路径匹配翻译      | 按路径注册翻译 handler，按 locale 获取翻译    |
| F-06-02 | LocalizedText 类型 | 数据库中存储多语言内容，按客户端 locale 自动选择最佳语言 |
| F-06-03 | 支持语言             | en, zh-CN, ja, es                |


### F-07 FFI 绑定


| 编号      | 需求          | 说明                                                           |
| ------- | ----------- | ------------------------------------------------------------ |
| F-07-01 | C API       | 11 个函数：create/free/get/emit/subscribe/unsubscribe/i18n/error |
| F-07-02 | 头文件         | `flux_ffi.h`，含 FluxBytes 结构体和 FluxChangeCallback 类型          |
| F-07-03 | 错误处理        | 线程局部错误存储，`flux_last_error()` 获取                              |
| F-07-04 | iOS 静态库     | 编译为 aarch64-apple-ios + aarch64-apple-ios-sim，打包 XCFramework |
| F-07-05 | Android 动态库 | 编译为 aarch64-linux-android 等（待实现）                             |


### F-08 客户端 SDK


| 编号      | 需求            | 说明                                                               |
| ------- | ------------- | ---------------------------------------------------------------- |
| F-08-01 | Swift 封装层     | FluxClient（get/emit/subscribe/i18n）+ FluxStore（ObservableObject） |
| F-08-02 | Dart 封装层      | FluxClient（emit/setLocale），缺 get/subscribe                       |
| F-08-03 | Rust HTTP 客户端 | ResourceClientT 类型安全 CRUD + FacetClientBase                      |


### F-09 DevOps


| 编号      | 需求       | 说明                                 |
| ------- | -------- | ---------------------------------- |
| F-09-01 | CI       | GitHub Actions：fmt + clippy + test |
| F-09-02 | Docker   | 多阶段构建 Dockerfile + docker-compose  |
| F-09-03 | iOS 构建脚本 | scripts/build-ios.sh → XCFramework |


---

## 第二部分：TwitterFlux 示例需求

### T-01 数据模型


| 编号      | 模型      | 字段                                                                                                   |
| ------- | ------- | ---------------------------------------------------------------------------------------------------- |
| T-01-01 | User    | id, username, password_hash, display_name, bio, avatar, follower_count, following_count, tweet_count |
| T-01-02 | Tweet   | id, author (NameUser), content, image_url, like_count, reply_count, reply_to                         |
| T-01-03 | Like    | id, user (NameUser), tweet (NameTweet)，组合键                                                           |
| T-01-04 | Follow  | id, follower (NameUser), followee (NameUser)，组合键                                                     |
| T-01-05 | Message | id, kind, sender, recipient, title (LocalizedText), body (LocalizedText), read                       |


### T-02 后端 API


| 编号      | 功能   | 路径                                    | 说明                |
| ------- | ---- | ------------------------------------- | ----------------- |
| T-02-01 | 登录   | POST /app/twitter/auth/login          | 用户名 + 密码，返回 JWT   |
| T-02-02 | 当前用户 | GET /app/twitter/me                   | 返回登录用户信息          |
| T-02-03 | 更新资料 | PUT /app/twitter/me/profile           | 含乐观锁              |
| T-02-04 | 修改密码 | PUT /app/twitter/me/password          |                   |
| T-02-05 | 时间线  | POST /app/twitter/timeline            | 关注者的推文，分页         |
| T-02-06 | 发推   | POST /app/twitter/tweets              | 含可选图片             |
| T-02-07 | 推文详情 | POST /app/twitter/tweets/{id}/detail  | 含回复列表             |
| T-02-08 | 点赞   | POST /app/twitter/tweets/{id}/like    | 幂等                |
| T-02-09 | 取消点赞 | DELETE /app/twitter/tweets/{id}/like  |                   |
| T-02-10 | 关注   | POST /app/twitter/users/{id}/follow   |                   |
| T-02-11 | 取消关注 | DELETE /app/twitter/users/{id}/follow |                   |
| T-02-12 | 用户主页 | POST /app/twitter/users/{id}/profile  | 含推文列表             |
| T-02-13 | 搜索   | POST /app/twitter/search              | 全文搜索用户 + 推文       |
| T-02-14 | 上传图片 | POST /app/twitter/upload              | 最大 5MB            |
| T-02-15 | 站内信  | POST /app/twitter/inbox               | 按 Accept-Language |
| T-02-16 | 标记已读 | POST /app/twitter/messages/{id}/read  |                   |


### T-03 BFF 状态


| 编号      | 路径            | 说明                           |
| ------- | ------------- | ---------------------------- |
| T-03-01 | auth/state    | 登录状态（phase, username, token） |
| T-03-02 | auth/login    | 登录请求 handler                 |
| T-03-03 | home/timeline | 时间线数据                        |
| T-03-04 | tweet/detail  | 推文详情数据                       |
| T-03-05 | user/profile  | 用户主页数据                       |


### T-04 iOS 示例 App（SwiftUI）


| 编号      | 页面   | 状态   | 调用的 API                                     |
| ------- | ---- | ---- | ------------------------------------------- |
| T-04-01 | 登录页  | TODO | emit("auth/login")                          |
| T-04-02 | 时间线  | TODO | get("timeline/feed"), emit("timeline/load") |
| T-04-03 | 推文详情 | TODO | emit("tweet/detail"), emit("tweet/like")    |
| T-04-04 | 个人主页 | TODO | emit("profile/load"), emit("user/follow")   |
| T-04-05 | 发推页  | TODO | emit("tweet/create")                        |
| T-04-06 | 搜索页  | TODO | emit("search/query")                        |
| T-04-07 | 站内信  | TODO | emit("inbox/load")                          |
| T-04-08 | 设置页  | TODO | setLocale(), emit("auth/logout")            |


### T-05 Flutter 示例 App


| 编号      | 页面  | 状态   | 说明       |
| ------- | --- | ---- | -------- |
| T-05-01 | 登录页 | TODO | 同 iOS 功能 |
| T-05-02 | 时间线 | TODO |          |
| T-05-03 | 搜索页 | TODO |          |
| T-05-04 | 站内信 | TODO |          |
| T-05-05 | 设置页 | TODO |          |


### T-06 种子数据


| 编号      | 数据  | 数量                                |
| ------- | --- | --------------------------------- |
| T-06-01 | 用户  | 5 个（alice, bob, carol, dave, eve） |
| T-06-02 | 推文  | 10 条 + 5 条回复                      |
| T-06-03 | 点赞  | 17 个                              |
| T-06-04 | 关注  | 10 个                              |
| T-06-05 | 站内信 | 3 条（多语言：en/zh-CN/ja/es）           |


---

## 约束

- 所有存储必须是嵌入式的（不依赖外部数据库服务）
- FFI 必须是 raw C API（不使用 UniFFI），一套接口服务 iOS + Flutter
- 框架代码由 AI Agent 编写，UI 代码由人类手写
- 示例 App 优先 iOS，其次 Flutter

