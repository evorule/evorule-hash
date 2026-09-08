# evorule-hash

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
> **当前版本**: v0.1.2（）。evorule-bundle / evorule-rule 直接依赖本仓；
> evorule-server 经 bundle 间接消费，并通过 `[patch.crates-io]` 钉到本地源。

## API

- `digest(&[u8]) -> String` —— 核心原语：`blake3::hash(bytes).to_hex()`，无前缀 64-hex
- `json_digest<T: Serialize>(&T) -> String` —— JSON 内容的 BLAKE3（无前缀 64-hex），镜像 reactor `content_hash`
- `prefixed(&str) -> String` —— `blake3:` 自描述前缀包装，供 bundle/entry 存储字段使用

## 哈希域注册表

生态内各哈希用途的域登记（2026-08-29 审计⑥后现状）。**不同域的哈希值不可互证**——任何校验逻辑必须与被校验值同域。

| 域 | 输入规范 | 前缀 | SSOT 路径 | 现状 |
|---|---|---|---|---|
| 审计链事实哈希 | TCB JsonValue 经 serde_json 紧凑序列化 | 无 | `evorule-reactor::hash`（冻结仓） | 基准 |
| 治理资产（bundle/条目指纹） | serde_json canonical 值 | `blake3:` | **本 crate** `json_digest`+`prefixed` | evorule-bundle / evorule-rule 已接入；server bundle API 经 bundle 间接接入 |
| 凭据（口令/API key/JWT） | PBKDF2-HMAC / SHA-256 | 无 | evorule-rule `src/auth` | 密码学标准做法，禁止与内容哈希字段混用 |
| 发布队列规则集哈希 | 自定义拼接（publish_service） | 无 | evorule-server `core/workspace` | **旁路待收口**（阶段 2 C1/C3） |
| 认知审计链（agent 自有） | 自定 canonical_json + 链步 | 无 | evorule-agent `src/cognition` | 独立域，公式自声明 |
| 会话锚 initial_content_hash | TCB JsonValue **Display**（带空格） | 无 | evorule-governance `session.rs` | **口径分叉待修**（阶段 2 C2）——与审计链事实哈希不可互证 |

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

## 许可证

**AGPL-3.0-or-later**（依据 DEC-2026-001 D-001-09 统一：审计链哈希是 EvoRule 差异化核心组件，统一为 AGPL 以保护商业价值）。本仓采用 EvoRule 双许可架构：闭源使用见 [DUAL_LICENSE.md](DUAL_LICENSE.md) / [FREE_COMMERCIAL_LICENSE.md](FREE_COMMERCIAL_LICENSE.md)（合格实体免费豁免）/ [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md)（付费）。`core_eval.json` 宪法为 **CC0-1.0**。商业许可咨询：evorulelab@gmail.com。

> 历史说明：v0.1.0–v0.1.2 曾以 AGPL 发布，v0.1.3 短暂以 Apache-2.0 发布；已发布的旧版本仍按原许可，本变更仅对新版本生效，已下载副本不受影响。
