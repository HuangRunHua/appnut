# PRD-0004: RBAC 细粒度权限系统

## 状态
草稿

## 背景

当前的 `JwtAuthenticator` 只验证"你是谁"（token 是否合法），不检查"你能做什么"——`permission` 参数被直接忽略。这意味着只要持有合法 JWT，就能访问所有 API，包括 Admin API。

几乎所有数据驱动型 App 都需要角色区分：
- 社交 App：普通用户 vs 管理员 vs 创作者
- 电商 App：买家 vs 卖家 vs 客服 vs 运营
- 内容平台：读者 vs 作者 vs 审核员 vs 管理员

框架需要一套通用的权限系统，让开发者能声明角色和权限，框架自动在 API 层做拦截。

## 需求

### 权限模型

1. **角色定义**：开发者能定义自己的角色集合（如 admin、user、creator、moderator）
2. **权限串**：沿用现有的 `{module}:{resource}:{action}` 格式（如 `twitter:tweet:create`、`twitter:user:delete`）
3. **角色-权限映射**：每个角色关联一组允许的权限串，支持通配符（如 `twitter:*:*` 表示 twitter 模块的所有权限）
4. **超级管理员**：有一个内置的超级角色，拥有所有权限（用于 Admin Dashboard）

### 认证集成

5. **JWT 携带角色信息**：用户登录后签发的 JWT 中包含角色标识
6. **自动拦截**：Admin API 和 Facet API 在执行前自动检查当前用户的角色是否拥有所需权限，无需每个 handler 手动检查
7. **`Authenticator` trait 兼容**：新的 RBAC 检查应作为 `Authenticator` 的增强实现，不破坏现有 trait 接口

### 开发者体验

8. **声明式权限**：开发者在 model 或 facet 定义中能声明所需权限，框架自动应用
9. **默认值**：如果开发者不配置权限，Admin API 默认要求 admin 角色，Facet API 默认要求已登录即可
10. **权限查询**：提供 API 让客户端查询当前用户拥有的权限列表（用于前端按钮/菜单的显示隐藏）

## 验收标准

- [ ] Admin API 从 AllowAll 切换到基于角色的权限检查
- [ ] 普通用户的 JWT 无法访问 Admin API
- [ ] 开发者能自定义角色和权限映射
- [ ] 权限串支持通配符匹配
- [ ] 在 TwitterFlux 示例中演示至少两种角色（admin + user）的权限差异
- [ ] 现有 Facet API 测试适配后仍全部通过

## 技术范围

- 涉及的 crate / 模块：`crates/support/core`（auth）、`crates/dsl/store`（admin router）、`crates/dsl/macro`（可能需要增加权限注解）
- 需要修改的文件：`crates/support/core/src/auth.rs`、`crates/dsl/store/src/admin.rs`
- 需要新增的文件：权限配置/角色定义相关模块
- 示例相关：`example/rust/server/` 需要适配 RBAC

## 约束

- `Authenticator` trait 的接口签名尽量保持兼容，避免所有下游都要改
- 不在本 PRD 中实现组织/租户级别的权限隔离（多租户是一个独立课题）
- 开发阶段可通过配置降级为 AllowAll，方便调试
