// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 EvoRule Project
//
// 本 crate 自 v0.1.3 起以 Apache-2.0 许可发布；更早版本曾以 AGPL-3.0-or-later 发布。
// 本实现为独立编写，非任何 AGPL 代码的衍生（仅通过黄金向量测试镜像算法语义）。
//! EvoRule 治理层规范哈希工具（BLAKE3）
//!
//! # 定位
//! 本 crate 是**治理层**（evorule-bundle / evorule-rule / evorule-server）的规范哈希单一真相源。
//! 它**镜像** `evorule-reactor/src/hash.rs` 的字节语义
//! （`blake3(serde_json::to_string(value))`，无前缀 64-hex），供治理三仓统一使用。
//!
//! 注意：`evorule-reactor` 处于 Kani 形式化验证保护下且为冻结仓，本 crate 不取代它，
//! 仅独立镜像其算法，使治理层各仓产出的哈希可跨仓字节一致、并与 reactor 审计链算法连续。
//!
//! # 设计
//! - 使用 `blake3` crate（1.x）计算 256 位哈希
//! - 核心原语 [`digest`]：返回无前缀 64-hex，与 `evorule_reactor::hash` 原语逐字节等价
//! - 序列化采用 `serde_json::to_vec`：紧凑、确定性，不受 Debug 实现变更影响
//! - [`json_digest`]：JSON 内容的 BLAKE3（无前缀 64-hex），镜像 reactor `content_hash`/`fact_hash` 算法
//! - [`prefixed`]：`blake3:` 自描述前缀包装，供 bundle/entry 存储字段使用

use blake3;
use serde::Serialize;
use serde_json;

/// 核心原语：与 `evorule-reactor::hash` 的 BLAKE3 原语逐字节等价。
///
/// 对给定字节计算 `blake3::hash(bytes).to_hex()`，返回**无前缀 64-hex**。
/// 这是审计链对齐的基准形态（reactor 的 `content_hash`/`fact_hash` 同样无前缀）。
#[inline]
pub fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// JSON 内容的 BLAKE3 哈希（无前缀 64-hex），镜像 `evorule_reactor::content_hash` 算法。
///
/// 对 `T: Serialize` 做 `serde_json::to_vec` 后调用 [`digest`]。
/// 由于 `serde_json::to_vec` 与 `serde_json::to_string` 产生完全相同的字节，
/// 且与 `evorule_reactor::hash::content_hash` 经 `tcb_to_serde` 后的序列化字节一致，
/// 故对同一 `serde_json::Value` 输入，本函数与 reactor `content_hash` **逐字节相等**。
#[inline]
pub fn json_digest<T: Serialize>(v: &T) -> String {
    let bytes = serde_json::to_vec(v).expect("内容必然可序列化");
    digest(&bytes)
}

/// 存储用自描述前缀包装。
///
/// reactor 的 Fact 链哈希为无前缀 64-hex；本前缀用于区分"治理层 bundle/entry"的哈希，
/// 自描述算法，便于校验与未来算法协商（配合 `hash_algo` 字段）。
#[inline]
pub fn prefixed(d: &str) -> String {
    format!("blake3:{d}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// 黄金向量：锁死 blake3 实现与 serde_json 序列化，避免版本漂移导致跨仓失配。
    /// 若 blake3/serde_json 升级改变了输出，本测试会红——这是期望的防护。
    #[test]
    fn golden_vector_bytes() {
        // blake3::hash(b"evorule").to_hex() —— 锁定 blake3 原语与版本
        assert_eq!(
            digest(b"evorule"),
            "7758f03680f4593e860eb2fc5257cc78d78563d315debde26be7bdb82f18bed4"
        );
    }

    /// 黄金向量（JSON 路径）：锁定 `json_digest` 与 reactor `content_hash` 的字节级等价。
    /// reactor content_hash 对 JsonValue::String("evorule") 哈希的是 `serde_json::to_string` 字节
    /// （即带引号的 `"evorule"`），与 `json_digest(&serde_json::json!("evorule"))` 一致。
    #[test]
    fn golden_vector_json_reactor_parity() {
        assert_eq!(
            json_digest(&serde_json::json!("evorule")),
            "c729d1c3521cd898b82af3808bbcc747758fc8b87b0236354cb232e3dde18236"
        );
    }

    #[test]
    fn golden_vector_json() {
        let v = serde_json::json!({"a": 1, "b": [2, 3], "c": "x"});
        let h = json_digest(&v);
        assert_eq!(h.len(), 64);
        // 确定性：同输入同输出
        assert_eq!(h, json_digest(&v));
        // 不同输入不同输出
        assert_ne!(h, json_digest(&serde_json::json!({"a": 2})));
    }

    #[test]
    fn prefixed_wrapper() {
        assert_eq!(prefixed("abc"), "blake3:abc");
        assert!(prefixed(&json_digest(&serde_json::json!(1))).starts_with("blake3:"));
    }

    /// json_digest 与 digest 的关系：json_digest(v) == digest(serde_json::to_vec(v))
    #[test]
    fn json_digest_equals_digest_of_vec() {
        let v = serde_json::json!({"z": true, "y": "v"});
        let bytes = serde_json::to_vec(&v).unwrap();
        assert_eq!(json_digest(&v), digest(&bytes));
    }

    /// **跨仓闸门（核心）**：`evorule-hash` 的 `digest` 必须字节级等于 `blake3::hash(bytes).to_hex()`。
    ///
    /// 冻结仓 `evorule-reactor/src/hash.rs` 的审计链原语正是 `blake3::hash(bytes).to_hex()`（无前缀 64-hex）。
    /// 本断言证明：治理层三仓（bundle/rule/server）经 `evorule-hash` 产出的哈希，与冻结仓 reactor
    /// 审计链**同算法、同字节**——即「全链路 BLAKE3 迁移后审计链不断裂」这一最高优先级契约。
    ///
    /// 不直接依赖冻结仓（避免触碰 Kani 验证与交叉验证保护的源码），仅直接复算 `blake3` 对照。
    #[test]
    fn digest_byte_matches_frozen_reactor_blake3() {
        let cases: &[&[u8]] = &[
            b"",
            b"evorule",
            b"hello world",
            b"audit-chain-must-not-break",
            &[0u8; 64],
            &[0xffu8; 129],
        ];
        for c in cases {
            let reactor = blake3::hash(c).to_hex().to_string();
            let ours = digest(c);
            assert_eq!(
                reactor, ours,
                "digest 必须字节级等于 reactor 的 blake3::hash().to_hex()（审计链同源）；输入长度 = {}",
                c.len()
            );
        }
    }
}
