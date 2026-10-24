// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 EvoRule Project
// This file is part of EvoRule, licensed under GNU Affero General Public License v3 or later.
//! 哈希域类型化收口（自 v0.2.0 起）。
//!
//! 本模块是 README「哈希域注册表」的**代码化正本**：每个域对应注册表一行，
//! 域集合是**封闭的**（sealed trait，crate 外不可实现——私造域在编译期被拦），
//! 不同域的哈希值是**不同的类型**——没有跨域转换，[`PartialEq`] 仅在同域间成立，
//! 「不同域的哈希值不可互证」由类型系统强制，不再只是文档纪律。
//!
//! # 域能力分层
//!
//! - **计算型域**（实现 [`Computable`]）：输入规范可由本 crate 原语
//!   （[`crate::digest`] / [`crate::json_digest`]）达成，提供 [`DomainHash::digest`]
//!   与 [`DomainHash::from_json`] 入口。
//! - **引用型域**（仅实现 [`Domain`]）：哈希公式在别处（凭据慢哈希、独立 crate 的
//!   链步、流式指纹等），本 crate 不实现其算法，只提供**值容器**——
//!   经 [`DomainHash::from_hex`] 的严格形态校验后持有，保证域隔离与前缀纪律。
//!
//! # 值形态纪律（前缀不混用）
//!
//! [`Domain::PREFIXED`] 声明各域存储形态：`true` = 必须带 `blake3:` 前缀，
//! `false` = 必须是无前缀 64-hex。[`DomainHash::from_hex`] 按此严格校验，
//! 形态错配（如给审计链域喂 `blake3:` 前缀值）即拒绝——与 B 族口径一致。
//!
//! # 边界说明
//!
//! 类型化保护的是**持有态**：[`DomainHash::as_hex`] / [`DomainHash::to_stored`]
//! 取出裸值后即离开保护面（存储字段本来就以裸字符串形态落库）。
//! 收口目标是消费侧传参/比较时的误用，不是消灭字符串形态。

use core::marker::PhantomData;
use serde::Serialize;

/// 域封闭集合的 sealed 根（crate 外不可名状、不可实现）。
mod sealed {
    pub trait Sealed {}
}

/// 哈希域：README「哈希域注册表」的代码化行。
///
/// 任何新增哈希用途必须先在本模块立域（新增 marker + impl），并同步 README
/// 注册表行——顺序反过来（先写调用后立域）会被类型系统拦住。
pub trait Domain: sealed::Sealed {
    /// 域名，与 README 注册表行首逐字一致（注册表对齐测试锁死）。
    const NAME: &'static str;
    /// 值形态纪律：`true` = 存储形态带 `blake3:` 前缀；`false` = 无前缀 64-hex。
    const PREFIXED: bool;
}

/// 计算型域：输入规范可由本 crate 原语达成，提供计算入口。
///
/// sealed——仅本模块声明的计算型域可实现；引用型域不得实现
/// （公式在别处的域不提供本 crate 的计算入口，防口径混装）。
pub trait Computable: Domain {}

/// 域哈希值校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidDomainHash {
    /// 值长度不是 64-hex 主体的长度。
    BadLength {
        /// 域名
        domain: &'static str,
        /// 剥离前缀后实际长度
        got: usize,
    },
    /// 主体含非 hex 字符。
    BadCharset {
        /// 域名
        domain: &'static str,
    },
    /// 值形态与域纪律不符（该带 `blake3:` 前缀的没带 / 不该带的带了）。
    PrefixMismatch {
        /// 域名
        domain: &'static str,
        /// 该域要求的存储形态
        expected: &'static str,
    },
}

impl core::fmt::Display for InvalidDomainHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadLength { domain, got } => {
                write!(f, "域「{domain}」哈希主体长度应为 64-hex，实际 {got}")
            }
            Self::BadCharset { domain } => {
                write!(f, "域「{domain}」哈希主体含非 hex 字符")
            }
            Self::PrefixMismatch { domain, expected } => {
                write!(
                    f,
                    "域「{domain}」值形态错配：要求 {expected}，各域形态不可混用"
                )
            }
        }
    }
}

impl std::error::Error for InvalidDomainHash {}

/// 域参数化哈希值：[`Domain`] 携带域身份，`PartialEq` 仅同域成立。
///
/// - 构造：[`DomainHash::from_hex`]（全域）；[`DomainHash::digest`] /
///   [`DomainHash::from_json`]（仅 [`Computable`] 域）
/// - 取用：[`DomainHash::as_hex`]（裸 hex）/ [`DomainHash::to_stored`]（域纪律存储形态）
/// - 跨域比较：`DomainHash<A>` 与 `DomainHash<B>` 是不同类型，直接比较即编译错——
///   互证在类型层面不可表达。
#[derive(Clone, Debug, Eq)]
pub struct DomainHash<D: Domain> {
    hex: String,
    _domain: PhantomData<fn() -> D>,
}

impl<D: Domain> PartialEq for DomainHash<D> {
    fn eq(&self, other: &Self) -> bool {
        self.hex == other.hex
    }
}

impl<D: Domain> core::hash::Hash for DomainHash<D> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.hex.hash(state);
    }
}

impl<D: Domain> DomainHash<D> {
    /// 引用构造：严格校验值形态（64-hex 主体 + 域前缀纪律）。
    ///
    /// 接受的形态由 [`Domain::PREFIXED`] 决定：前缀域收 `blake3:<64hex>`，
    /// 无前缀域收裸 64-hex；错配即 [`InvalidDomainHash::PrefixMismatch`]。
    pub fn from_hex(value: &str) -> Result<Self, InvalidDomainHash> {
        let expected: &'static str = if D::PREFIXED {
            "`blake3:` 前缀 + 64-hex"
        } else {
            "无前缀 64-hex"
        };
        let raw = if D::PREFIXED {
            value
                .strip_prefix("blake3:")
                .ok_or(InvalidDomainHash::PrefixMismatch {
                    domain: D::NAME,
                    expected,
                })?
        } else {
            if value.starts_with("blake3:") {
                return Err(InvalidDomainHash::PrefixMismatch {
                    domain: D::NAME,
                    expected,
                });
            }
            value
        };
        if raw.len() != 64 {
            return Err(InvalidDomainHash::BadLength {
                domain: D::NAME,
                got: raw.len(),
            });
        }
        if !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(InvalidDomainHash::BadCharset { domain: D::NAME });
        }
        Ok(Self {
            hex: raw.to_owned(),
            _domain: PhantomData,
        })
    }

    /// 域名（注册表行标识）。
    #[must_use]
    pub fn domain_name() -> &'static str {
        D::NAME
    }

    /// 裸 hex（无前缀 64-hex）。取出即离开类型保护面。
    #[must_use]
    pub fn as_hex(&self) -> &str {
        &self.hex
    }

    /// 域纪律存储形态：前缀域输出 `blake3:<hex>`，无前缀域输出裸 hex。
    #[must_use]
    pub fn to_stored(&self) -> String {
        if D::PREFIXED {
            format!("blake3:{}", self.hex)
        } else {
            self.hex.clone()
        }
    }
}

impl<D: Computable> DomainHash<D> {
    /// 计算构造：字节直哈希（本 crate 原语 [`crate::digest`]，域内计算）。
    #[must_use]
    pub fn digest(bytes: &[u8]) -> Self {
        Self {
            hex: crate::digest(bytes),
            _domain: PhantomData,
        }
    }

    /// 计算构造：JSON 内容哈希（本 crate 原语 [`crate::json_digest`]）。
    ///
    /// 注意 TCB `JsonValue` 未实现 `Serialize`：调用方须先转换为
    /// `serde_json::Value` 后传入——转换逻辑本身是口径的一部分
    /// （统一转换契约见 README「转换接缝声明」）。
    #[must_use]
    pub fn from_json<T: Serialize>(v: &T) -> Self {
        Self {
            hex: crate::json_digest(v),
            _domain: PhantomData,
        }
    }
}

// ---------------------------------------------------------------------------
// 域清单：与 README「哈希域注册表」逐行对齐（注册表对齐测试锁死 NAME/PREFIXED）。
// 新增域 = 新增 marker + impl（Domain，计算型另加 Computable）+ 对齐测试加行 + README 加行。
// ---------------------------------------------------------------------------

/// 审计链事实哈希：TCB JsonValue 经 serde_json 紧凑序列化，无前缀。
/// SSOT = 冻结仓 reactor `hash`（本 crate `digest` 逐字节镜像）。
/// 与其他任何域不可互证。
pub struct AuditChainFacts;
impl Domain for AuditChainFacts {
    const NAME: &'static str = "审计链事实哈希";
    const PREFIXED: bool = false;
}
impl Computable for AuditChainFacts {}

/// 治理资产（bundle/条目指纹）：serde_json canonical 值，存储带 `blake3:` 前缀。
/// SSOT = 本 crate `json_digest` + `prefixed`（bundle/rule 已接入）。
pub struct GovernanceAsset;
impl Domain for GovernanceAsset {
    const NAME: &'static str = "治理资产";
    const PREFIXED: bool = true;
}
impl Computable for GovernanceAsset {}

/// 凭据（口令/API key/JWT）：PBKDF2-HMAC / SHA-256，无前缀。
/// 公式在 evorule-rule `src/auth`——本 crate **不实现**凭据算法（引用型域），
/// 禁止与内容哈希字段混用。
pub struct Credentials;
impl Domain for Credentials {
    const NAME: &'static str = "凭据";
    const PREFIXED: bool = false;
}

/// 平台 token 库存哈希：64-hex 随机 token（≥512-bit 熵）字节直哈希 blake3，无前缀。
/// 凭据域高熵子形态：查表哈希非口令慢哈希，仅限 ≥256-bit 高熵随机 token，
/// 低熵口令禁用本域（用 [`Credentials`] 面）。
pub struct TokenInventory;
impl Domain for TokenInventory {
    const NAME: &'static str = "平台 token 库存哈希";
    const PREFIXED: bool = false;
}
impl Computable for TokenInventory {}

/// 发布队列规则集哈希：自定义拼接（publish_service），无前缀。
/// **旁路待收口**（阶段 2 C1/C3）：公式未收敛前仅立值容器，不提供计算入口。
pub struct PublishQueueRuleset;
impl Domain for PublishQueueRuleset {
    const NAME: &'static str = "发布队列规则集哈希";
    const PREFIXED: bool = false;
}

/// 认知审计链（agent 自有）：自定 canonical_json + 链步，无前缀。
/// 公式在 agent 认知模块（独立域，自声明）——引用型域。
pub struct CognitionAuditChain;
impl Domain for CognitionAuditChain {
    const NAME: &'static str = "认知审计链";
    const PREFIXED: bool = false;
}

/// 会话锚 initial_content_hash：TCB JsonValue **Display**（带空格）字节，无前缀。
/// **口径分叉待修**（阶段 2 C2）：与审计链事实哈希不可互证——正因分叉，
/// 本域与 [`AuditChainFacts`] 是两个独立类型，杜绝混用。
pub struct SessionAnchor;
impl Domain for SessionAnchor {
    const NAME: &'static str = "会话锚 initial_content_hash";
    const PREFIXED: bool = false;
}
impl Computable for SessionAnchor {}

/// 上下文快照链：canonical JSON(skeleton) 等（BTreeMap 键序、浮点拒收、整数纪律）。
/// 公式在 evorule-llm-context-inspector（独立域）——引用型域。
pub struct ContextSnapshot;
impl Domain for ContextSnapshot {
    const NAME: &'static str = "上下文快照链";
    const PREFIXED: bool = false;
}

/// skill 注册账本内容哈希：SKILL.md 文件字节直哈希，存储带 `blake3:` 前缀。
/// 审批时刻钉哈希，会话创建重算、失配自动降级。
pub struct SkillLedger;
impl Domain for SkillLedger {
    const NAME: &'static str = "skill 注册账本内容哈希";
    const PREFIXED: bool = true;
}
impl Computable for SkillLedger {}

/// 市场资产内容哈希（模板/wasm 分发）：分发文件字节直哈希，**裸 64-hex 无前缀**
/// （消费方 `blake3::hash(..).to_string()` 产出裸形态，与 wasm 清单的前缀形态不同）。
/// 分发完整性校验（下载/装载侧重算比对），与治理哈希不可互证。
pub struct MarketplaceContent;
impl Domain for MarketplaceContent {
    const NAME: &'static str = "市场资产内容哈希";
    const PREFIXED: bool = false;
}
impl Computable for MarketplaceContent {}

/// wasm 分发完整性链：`.wasm` 文件字节直哈希（清单 `blake3` 字段），存储带 `blake3:` 前缀。
/// 签名链 sig→清单字节→blake3→wasm，一签锚两文件；与治理资产同形态不同域，不可互证。
pub struct WasmIntegrity;
impl Domain for WasmIntegrity {
    const NAME: &'static str = "wasm 分发完整性链";
    const PREFIXED: bool = true;
}
impl Computable for WasmIntegrity {}

/// 规则集版本哈希：`VERSION_HASH_DOMAIN` 分离常数 + 流式 Hasher 多段输入，无前缀。
/// 流式公式在消费侧（版本分桶指纹，非验证锚点）——引用型域。
pub struct RulesetVersion;
impl Domain for RulesetVersion {
    const NAME: &'static str = "规则集版本哈希";
    const PREFIXED: bool = false;
}

/// 测试确定性快照哈希：内核/状态 Debug 串直哈希（同构建确定），无前缀。
/// 仅测试自证比对，不入链不入库。
pub struct TestSnapshot;
impl Domain for TestSnapshot {
    const NAME: &'static str = "测试确定性快照哈希";
    const PREFIXED: bool = false;
}
impl Computable for TestSnapshot {}

/// 域封闭集合落地：全部 marker 收进 sealed 根（crate 外不可实现新域）；
/// 顺带给 marker 实现 Debug（输出域名，使 `DomainHash<D>` 的 derive Debug 无条件可用）。
macro_rules! impl_sealed {
    ($($t:ty),* $(,)?) => {
        $(
            impl sealed::Sealed for $t {}
            impl core::fmt::Debug for $t {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str(<$t as Domain>::NAME)
                }
            }
        )*
    };
}
impl_sealed!(
    AuditChainFacts,
    GovernanceAsset,
    Credentials,
    TokenInventory,
    PublishQueueRuleset,
    CognitionAuditChain,
    SessionAnchor,
    ContextSnapshot,
    SkillLedger,
    MarketplaceContent,
    WasmIntegrity,
    RulesetVersion,
    TestSnapshot,
);

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    /// 注册表对齐锁：逐域断言 NAME/PREFIXED 与 README「哈希域注册表」行一致。
    /// 改名/改形态必须三处同步：marker、本测试、README 行。
    #[test]
    fn registry_alignment() {
        struct Row {
            name: &'static str,
            prefixed: bool,
        }
        fn check<D: Domain>(row: Row) {
            assert_eq!(D::NAME, row.name, "域 NAME 与注册表行漂移");
            assert_eq!(D::PREFIXED, row.prefixed, "域 PREFIXED 与注册表行漂移");
        }
        // 与 README 域注册表行序一致
        check::<AuditChainFacts>(Row {
            name: "审计链事实哈希",
            prefixed: false,
        });
        check::<GovernanceAsset>(Row {
            name: "治理资产",
            prefixed: true,
        });
        check::<Credentials>(Row {
            name: "凭据",
            prefixed: false,
        });
        check::<TokenInventory>(Row {
            name: "平台 token 库存哈希",
            prefixed: false,
        });
        check::<PublishQueueRuleset>(Row {
            name: "发布队列规则集哈希",
            prefixed: false,
        });
        check::<CognitionAuditChain>(Row {
            name: "认知审计链",
            prefixed: false,
        });
        check::<SessionAnchor>(Row {
            name: "会话锚 initial_content_hash",
            prefixed: false,
        });
        check::<ContextSnapshot>(Row {
            name: "上下文快照链",
            prefixed: false,
        });
        check::<SkillLedger>(Row {
            name: "skill 注册账本内容哈希",
            prefixed: true,
        });
        check::<MarketplaceContent>(Row {
            name: "市场资产内容哈希",
            prefixed: false,
        });
        check::<WasmIntegrity>(Row {
            name: "wasm 分发完整性链",
            prefixed: true,
        });
        check::<RulesetVersion>(Row {
            name: "规则集版本哈希",
            prefixed: false,
        });
        check::<TestSnapshot>(Row {
            name: "测试确定性快照哈希",
            prefixed: false,
        });
    }

    /// 前缀纪律（运行时面）：形态错配即拒，`blake3:` 前缀不混用。
    #[test]
    fn prefix_discipline() {
        let hex = "7758f03680f4593e860eb2fc5257cc78d78563d315debde26be7bdb82f18bed4";
        // 无前缀域拒收 blake3: 前缀值
        let e = DomainHash::<AuditChainFacts>::from_hex(&format!("blake3:{hex}"))
            .expect_err("无前缀域必须拒收前缀值");
        assert!(matches!(e, InvalidDomainHash::PrefixMismatch { .. }));
        // 前缀域拒收裸 hex
        let e = DomainHash::<GovernanceAsset>::from_hex(hex).expect_err("前缀域必须拒收裸值");
        assert!(matches!(e, InvalidDomainHash::PrefixMismatch { .. }));
        // 各自正确形态通过，to_stored 归一到域纪律形态
        let a = DomainHash::<AuditChainFacts>::from_hex(hex).unwrap();
        assert_eq!(a.as_hex(), hex);
        assert_eq!(a.to_stored(), hex);
        let g = DomainHash::<GovernanceAsset>::from_hex(&format!("blake3:{hex}")).unwrap();
        assert_eq!(g.as_hex(), hex);
        assert_eq!(g.to_stored(), format!("blake3:{hex}"));
    }

    /// 值校验：长度/字符集。
    #[test]
    fn value_validation() {
        let e = DomainHash::<AuditChainFacts>::from_hex("abc").expect_err("短值必须拒");
        assert!(matches!(e, InvalidDomainHash::BadLength { got: 3, .. }));
        let bad = "zz58f03680f4593e860eb2fc5257cc78d78563d315debde26be7bdb82f18bed4";
        let e = DomainHash::<AuditChainFacts>::from_hex(bad).expect_err("非 hex 必须拒");
        assert!(matches!(e, InvalidDomainHash::BadCharset { .. }));
    }

    /// 黄金向量域化：计算型域入口与既有黄金向量逐字节一致（字节语义零变化）。
    #[test]
    fn golden_vectors_through_domains() {
        let j = serde_json::json!("evorule");
        let a = DomainHash::<AuditChainFacts>::from_json(&j);
        assert_eq!(
            a.as_hex(),
            "c729d1c3521cd898b82af3808bbcc747758fc8b87b0236354cb232e3dde18236"
        );
        let g = DomainHash::<GovernanceAsset>::digest(b"evorule");
        assert_eq!(
            g.to_stored(),
            "blake3:7758f03680f4593e860eb2fc5257cc78d78563d315debde26be7bdb82f18bed4"
        );
    }

    /// 同域相等/不等 + 往返（digest → to_stored → from_hex 同值）。
    #[test]
    fn same_domain_eq_and_roundtrip() {
        let a = DomainHash::<MarketplaceContent>::digest(b"asset.bin");
        let b = DomainHash::<MarketplaceContent>::from_hex(&a.to_stored()).unwrap();
        assert_eq!(a, b);
        assert_ne!(
            a,
            DomainHash::<MarketplaceContent>::digest(b"other.bin"),
            "同域不同输入必须不等"
        );
        // 引用型域：仅值容器（公式在别处，本 crate 不算）
        let c = DomainHash::<Credentials>::from_hex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )
        .unwrap();
        assert_eq!(
            c.as_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(DomainHash::<Credentials>::domain_name(), "凭据");
    }
}
