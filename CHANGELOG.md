<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->
<!-- Copyright (C) 2026 EvoRule Project -->

# 更新日志

本文件记录 evorule-hash 的显著变更。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循[语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

## [0.1.4] - 2026-09-22

### 变更

- **许可证回归 AGPL-3.0-or-later**（依生态许可统一裁定：审计链哈希是差异化核心组件，统一为 AGPL 以保护商业价值）。历史轨迹：v0.1.0–v0.1.2 AGPL → v0.1.3 Apache-2.0 → v0.1.4 起回归 AGPL
- crates.io 上已发布的 v0.1.3（Apache-2.0）许可不可改，消费方钉版时注意许可口径；本版本号承载 AGPL 口径发布

## [0.1.3] - 2026-08-29

### 变更

- **许可证变更：AGPL-3.0-or-later → Apache-2.0**（自本版本起生效）。本 crate 为独立实现、非任何 AGPL 代码的衍生（独立实现声明见 NOTICE.md）；Apache-2.0 自带专利授权条款，利于治理层契约库的生态接入
- 历史版本 v0.1.0–v0.1.2 仍适用 AGPL-3.0-or-later（已发布版本不可撤回）

### 文档

- README 增补"哈希域注册表"：登记生态内 6 个哈希域（审计链事实哈希 / 治理资产 / 凭据 / 发布队列规则集哈希 / 认知审计链 / 会话锚），声明不同域哈希值不可互证
- README 增补 TCB `JsonValue` 转换接缝声明（`json_digest` 仅接受 `serde_json::Serialize`，统一转换契约为生态待办）
- 建立版本控制基线与 CHANGELOG

## [0.1.2] - 2026-08-25

已发布至 crates.io。

### 变更

- crate 打包材料完备（LICENSE / NOTICE / README 纳入发布包）

## [0.1.1] - 2026-08-25

已发布至 crates.io。

### 变更

- Cargo.toml 元数据补全（homepage / repository 指向 Gitee 仓）

## [0.1.0] - 2026-08-25

已发布至 crates.io。

### 新增

- 初始版本：治理资产规范哈希 crate（BLAKE3）
- `json_digest`（serde_json canonical 值摘要）与 `prefixed`（`blake3:` 前缀格式）
- 黄金向量测试：独立镜像 `evorule-reactor` 哈希算法，用真实 Reactor 快照锁死字节级等价
- 定位：evorule-bundle / evorule-rule 的单一哈希真相源；`evorule-reactor` 处于 Kani 形式化验证保护下且为冻结仓，本 crate 不取代它，仅独立镜像

[0.1.3]: https://gitee.com/evorule/evorule-hash/compare/v0.1.2...v0.1.3
[0.1.2]: https://gitee.com/evorule/evorule-hash/compare/v0.1.1...v0.1.2
[0.1.1]: https://gitee.com/evorule/evorule-hash/compare/v0.1.0...v0.1.1
[0.1.0]: https://gitee.com/evorule/evorule-hash/releases/tag/v0.1.0
