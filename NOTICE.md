<!--
  Copyright 2026 EvoRule Project
  SPDX-License-Identifier: Apache-2.0
-->

# EvoRule Hash — 声明

**版权所有 (c) 2026 EvoRule Project**

本 crate（`evorule-hash`）是 EvoRule 生态的一部分，提供治理层规范哈希工具（BLAKE3）。

## 协议

| 资产 | 协议 | 说明 |
|---|---|---|
| **代码**（v0.1.3 起） | Apache-2.0 | 详见 [LICENSE](LICENSE) |
| 代码（v0.1.0–v0.1.2 历史版本） | AGPL-3.0-or-later | 已发布版本不可撤回，历史版本仍适用原许可 |

## 独立实现声明

本 crate 为**独立编写**，非任何 AGPL 代码（含 `evorule-reactor`）的衍生作品：
仅通过黄金向量测试镜像 `evorule-reactor::hash` 的**算法语义**
（`blake3(serde_json 序列化字节)`，无前缀 64-hex），代码表达完全独立。
算法语义本身不受版权保护；两仓共享的仅为对 `blake3` crate 的标准惯用法调用。

## 定位

治理层（evorule-bundle / evorule-rule / evorule-server）的规范哈希**单一真相源**。
镜像冻结仓 `evorule-reactor/src/hash.rs` 的字节语义，使各仓产出的哈希可跨仓字节一致，
并与 reactor 审计链算法连续。不取代冻结仓，仅独立镜像其算法（含黄金向量测试锁死字节级等价）。

## 设计原则

- 确定性：相同输入永远产生相同哈希
- 单一真相源：治理层各仓共用同一哈希实现，避免口径漂移
- 零隐藏逻辑：公开 API 即全部能力（digest / json_digest / prefixed）

## 联系信息

- **项目**: EvoRule — 反应式执行引擎
- **作者**: EvoRule Project
- **邮箱**: <evorulelab@gmail.com>
- **组织**: [EvoRule Lab](https://gitee.com/evorule)
- **Gitee**: <https://gitee.com/evorule/evorule-hash>
