# evorule-hash

> **仓库角色**：本仓库是 **GitHub 镜像**。权威源（primary repo）在 **Gitee**：[https://gitee.com/evorule/evorule-hash](https://gitee.com/evorule/evorule-hash)。
> Issue / PR 请提交到 Gitee，GitHub 侧仅供镜像与推广展示。


[![CI](https://github.com/evorule/evorule-hash/actions/workflows/ci.yml/badge.svg)](https://github.com/evorule/evorule-hash/actions/workflows/ci.yml)

EvoRule 治理层规范哈希工具（BLAKE3）。

## 定位

治理层（evorule-bundle / evorule-rule / evorule-server）的规范哈希**单一真相源**。
镜像 `evorule-reactor/src/hash.rs` 的字节语义
（`blake3(serde_json::to_string(value))`，无前缀 64-hex），供治理三仓统一使用，
使各仓产出的哈希可跨仓字节一致，并与 reactor 审计链算法连续。

> **注意**: `evorule-reactor` 处于 Kani 形式化验证保护下且为冻结仓，本 crate 不取代它，
> 仅独立镜像其算法（含黄金向量测试锁死字节级等价）。
>
> **当前版本**: v0.2.0（域类型化收口）。evorule-bundle / evorule-rule / evorule-server 直接依赖本仓
> （三仓依赖 req 全等，集成测试版本闸校验不低于 0.1.4；下游升版随各自自然批次——v0.2.0 对
> v0.1.4 为纯 additive，原语面字节语义零差异）；server 对 evorule-bundle 另有
> `[patch.crates-io]` 本地联调通道（gitignored，不进 CI）。

## API

- `digest(&[u8]) -> String` —— 核心原语：`blake3::hash(bytes).to_hex()`，无前缀 64-hex
- `json_digest<T: Serialize>(&T) -> String` —— JSON 内容的 BLAKE3（无前缀 64-hex），镜像 reactor `content_hash`
- `prefixed(&str) -> String` —— `blake3:` 自描述前缀包装，供 bundle/entry 存储字段使用

## 哈希域注册表

生态内各哈希用途的域登记（2026-08-29 审计⑥后现状；**自 v0.2.0 起代码化正本=`src/domains.rs`**——域为封闭集合（sealed），跨域不可互证由类型系统强制，前缀形态纪律在装载时校验；新增域须代码与下表两处同步）。**不同域的哈希值不可互证**——任何校验逻辑必须与被校验值同域。

| 域 | 输入规范 | 前缀 | SSOT 路径 | 现状 |
|---|---|---|---|---|
| 审计链事实哈希 | TCB JsonValue 经 serde_json 紧凑序列化 | 无 | `evorule-reactor::hash`（冻结仓） | 基准；server 离线重建验证点=evorule-server `src/api/audit_archive.rs`（同公式重算，2026-10-24 复核立档） |
| 治理资产（bundle/条目指纹） | serde_json canonical 值 | `blake3:` | **本 crate** `json_digest`+`prefixed` | evorule-bundle / evorule-rule 已接入；server bundle API 经 bundle 间接接入 |
| 凭据（口令/API key/JWT） | PBKDF2-HMAC / SHA-256 | 无 | evorule-rule `src/auth` | 密码学标准做法，禁止与内容哈希字段混用 |
| 平台 token 库存哈希 | 64-hex 随机 token（512-bit 熵）字节直哈希 blake3 | 无（hex） | evorule-server `src/api/platform_auth.rs` | 凭据域高熵子形态（2026-10-24 登记）：查表哈希非口令慢哈希，**仅限 ≥256-bit 高熵随机 token**，低熵口令禁用本形态 |
| 发布队列规则集哈希 | 自定义拼接（publish_service） | 无 | evorule-server `core/workspace` | **旁路待收口**（阶段 2 C1/C3） |
| 认知审计链（agent 自有） | 自定 canonical_json + 链步 | 无 | agent 认知模块 `src/cognition` | 独立域，公式自声明 |
| 会话锚 initial_content_hash | TCB JsonValue **Display**（带空格） | 无 | evorule-governance `session.rs` | **口径分叉待修**（阶段 2 C2）——与审计链事实哈希不可互证 |
| 上下文快照链（evorule-llm-context-inspector） | canonical JSON(skeleton)（BTreeMap 键序、浮点拒收、整数纪律）；skeleton_hash 与 observation 哈希；`content_hash` 例外=原文直哈希（非链验证锚点，可含 provider 原生浮点） | 无 | evorule-llm-context-inspector（`D:\evorule-context-inspector`） | 独立域（2026-10-02 登记，实施文档 v3 决策 4）；链步=复用 reactor `chain_step`，genesis=`"genesis"`；与审计链事实哈希不可互证 |
| skill 注册账本内容哈希（evo-agent skills 装配） | SKILL.md 文件字节直哈希（`digest`） | `blake3:` | evo-agent `src/agent/skill_store.rs` | 独立域（2026-10-03 登记）；审批时刻钉哈希，会话创建重算、失配自动降级；与审计链事实哈希不可互证 |
| 市场资产内容哈希（模板/wasm 分发） | 分发文件字节直哈希（`blake3::hash(..).to_string()` 即 `blake3:` 前缀形态） | `blake3:` | evorule-server `src/api/marketplace.rs` | 独立域（2026-10-24 登记）：分发完整性校验（下载/装载侧重算比对），与治理哈希不可互证 |
| wasm 分发完整性链 | `.wasm` 文件字节直哈希（清单 `blake3` 字段；签名链 sig→清单字节→blake3→wasm 一签锚两文件） | `blake3:` | wasm-host `src/integrity.rs` | 独立域（2026-10-24 登记）：与 bundle audit.content_hash 同形态不同域，不可互证 |
| 规则集版本哈希 | `VERSION_HASH_DOMAIN` 分离常数 + 规则数 + 各「规则序列化串\|来源标签」流式 Hasher | 无（hex） | evorule-server `src/api/hit_stats.rs` | 独立域（2026-10-24 登记）：版本分桶指纹（来源变化换版分桶），非验证锚点 |
| 测试确定性快照哈希 | 内核/状态 Debug 串直哈希（同构建确定） | 无（hex） | 各仓 tests（physics-services `test_thermal_determinism` 等） | 独立域（2026-10-24 登记）：仅测试自证比对，不入链不入库 |

### 转换接缝声明

`json_digest` 仅接受 `serde_json::Serialize` 输入。TCB `JsonValue`（evorule-tcb）**未实现 Serialize**，
调用方须先转换为 `serde_json::Value` 后方可入域——转换逻辑本身是口径的一部分，
统一转换契约（对齐 reactor `tcb_to_serde`）为阶段 2 待办（C2/C11）。

## 使用

```toml
[dependencies]
evorule-hash = "0.1"
```

```rust
use evorule_hash::{digest, json_digest, prefixed};

let h = digest(b"evorule");
let j = json_digest(&serde_json::json!({"a": 1}));
let s = prefixed(&h);
```

### 域类型化（v0.2.0 起）

```rust
use evorule_hash::domains::{AuditChainFacts, GovernanceAsset};
use evorule_hash::DomainHash;

// 计算型域：输入规范由本 crate 原语达成
let g = DomainHash::<GovernanceAsset>::digest(b"bytes");
g.to_stored();          // "blake3:<hex>"（域纪律存储形态）
let a = DomainHash::<AuditChainFacts>::from_json(&serde_json::json!({"x": 1}));

// 引用构造：严格校验 64-hex 主体与域前缀纪律，形态错配即拒
let v = DomainHash::<GovernanceAsset>::from_hex(&g.to_stored())?;

// 跨域不可互证 = 编译期：DomainHash<AuditChainFacts> 与 DomainHash<GovernanceAsset>
// 是不同类型，没有跨域转换，PartialEq 仅同域成立。
// 引用型域（凭据/认知链/快照链/流式指纹等公式在别处的域）只有 from_hex 值容器能力。
```

## 许可证

**AGPL-3.0-or-later**（依据 DEC-2026-001 D-001-09 统一：审计链哈希是 EvoRule 差异化核心组件，统一为 AGPL 以保护商业价值）。本仓采用 EvoRule 双许可架构：闭源使用见 [DUAL_LICENSE.md](DUAL_LICENSE.md) / [FREE_COMMERCIAL_LICENSE.md](FREE_COMMERCIAL_LICENSE.md)（合格实体免费豁免）/ [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md)（付费）。`core_eval.json` 宪法为 **CC0-1.0**。商业许可咨询：evorulelab@gmail.com。

> 历史说明：v0.1.0–v0.1.2 曾以 AGPL 发布，v0.1.3 短暂以 Apache-2.0 发布；自 v0.1.4 起依前述统一裁定**回归 AGPL-3.0-or-later**。已发布的旧版本仍按原许可（crates.io 已发布版本许可不可改），本变更仅对新版本生效，已下载副本不受影响；消费方钉版时注意许可口径。
