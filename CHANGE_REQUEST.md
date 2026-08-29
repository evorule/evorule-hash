# 变更审查表 (Change Request)

> **版本**: 2.0（适配独立库仓）
> **用途**: 本仓变更治理的强制审查记录
> **规则**: 本仓是**机制层**库 crate，只承载通用基础能力（哈希原语），业务策略变更必须留在应用层仓（evorule-server 等）实现。

---

## 1. 基本信息

| 字段 | 值 |
|------|------|
| **变更 ID** | CR-20260826-001 |
| **变更标题** | 开仓并首次发布（evorule-hash 规范哈希单一真相源） |
| **提交人** | EvoRule Team |
| **提交日期** | 2026-08-26 |
| **审查状态** | 已批准 |

## 2. 变更层级判定（必填）

**本次变更属于**: ✅ **机制层 (Mechanism)**

### 判定理由

```
- 提供通用哈希原语（digest / json_digest / prefixed），不包含任何特定业务语义
- 可被治理层任意场景无差别复用（bundle / rule / server）
- 不依赖冻结仓 evorule-reactor 源码，仅镜像其字节语义，黄金向量测试锁死字节级等价
```

### 机制层判定标准检查

**✅ 机制层变更的特征**:
- [x] 提供通用基础设施能力（BLAKE3 哈希原语）
- [x] 不包含任何特定业务语义
- [x] 可被任何业务场景无差别复用
- [x] 保持确定性（无时间 / 随机数 / 浮点 / 非确定性容器）

## 3. 变更分类

- **变更类型**: A - 新增机制
- **影响模块**: evorule-hash

## 4. 变更详情

### 4.1 变更理由

治理层（bundle / rule / server）需要跨仓字节一致的规范哈希，避免各仓独立实现导致口径漂移，
故开独立仓提供 BLAKE3 哈希单一真相源。

### 4.2 变更范围

- `src/lib.rs`：新增 `digest` / `json_digest` / `prefixed` 公开原语
- 黄金向量测试：锁定与 `evorule-reactor::hash` 的字节级等价（跨仓闸门）

### 4.3 破坏性分析

无破坏性变更（首次开仓）。

### 4.4 测试计划

- [x] 黄金向量（字节路径 / JSON 路径 / 跨仓闸门）
- [x] 确定性 / 前缀包装 / json_digest≡digest∘to_vec 单元测试

## 5. 审查清单

### 层级审查
- [x] 变更层级声明为"机制层"
- [x] 代码中无策略层反模式（硬编码业务字面量）

### 技术审查
- [x] 保持确定性（deterministic）
- [x] 无 unsafe / 无 I/O / 无系统时间

### 文档审查
- [x] README.md / NOTICE.md / LICENSE / CHANGE_REQUEST.md 齐备
- [x] Cargo.toml 元数据完整（description / license / repository / homepage / readme / rust-version）

---

> **注意**: 本仓为机制层库 crate，后续每次修改都需要同步更新此文件。
