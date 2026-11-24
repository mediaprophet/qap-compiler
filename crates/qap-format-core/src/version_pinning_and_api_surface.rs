//! `version_pinning_and_api_surface`:
//!
//! Granular Version Pinning, SemVer / 4-Segment Minor-Update Tolerance, and Explicit API Surface
//! Contract Diagnostics for `.qap` (Qualia Application Package) compilation and host execution.
//!
//! Design principles:
//! 1. **Know What It Was Designed For (`pinned_designed_version` + `abi_or_wire_epoch`)**:
//!    Every `.qap` records the exact ecosystem component versions, ABI/container epochs
//!    (`NQuin48-Q42-v2`, `HOLO-0x0306-10D-v2`, `VibeScript-0.1-0.2-ABI`), and optional source commit
//!    pins it was authored and verified against.
//! 2. **Minor & Patch Updates Do Not Disaffect Compilation (`[min_compatible_version ..= max_compatible_version]`)**:
//!    When a user's host environment has minor or patch updates (e.g. `0.0.41` -> `0.0.41.25` or `0.0.42`)
//!    within the declared compatible range and the required capability API surface contracts remain
//!    satisfied, `qapc` compiles and activates the `.qap` cleanly under
//!    `CompatibilityStatus::CompatibleMinorOrPatchDrift` while logging the exact version delta.
//! 3. **Pinpoint Diagnostics When an API Surface Changes (`CapabilitySurfaceContract`)**:
//!    Each required capability (`capability.invoke` target) declares its expected input parameter fields,
//!    output return fields, owning component, and deterministic surface hash. If a future host version
//!    renames, removes, or alters a capability's parameters or return structure, the compatibility report
//!    identifies the exact capability, the version it was designed for, the missing/changed fields, and
//!    concrete migration guidance.

use crate::{compute_entry_digest32, hex_encode, q_hash_u64, NQuin48};
use serde::{Deserialize, Serialize};

/// 4-Segment Qualia / Webizen Ecosystem Version (`major.minor.patch.build`, e.g. `0.0.41` or `0.0.41.20`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EcosystemVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
}

impl EcosystemVersion {
    pub const fn new(major: u32, minor: u32, patch: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
        }
    }

    /// Parses `"0.0.41"`, `"0.0.41.20"`, or `"v0.0.41-dev"` into an `EcosystemVersion`.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let cleaned = raw
            .trim()
            .trim_start_matches('v')
            .split('-')
            .next()
            .unwrap_or("")
            .trim();
        if cleaned.is_empty() {
            return Err(format!("Empty version string: '{raw}'"));
        }
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.is_empty() || parts.len() > 4 {
            return Err(format!(
                "Invalid ecosystem version '{raw}': expected 1 to 4 numeric segments"
            ));
        }
        let parse_seg = |idx: usize| -> Result<u32, String> {
            match parts.get(idx) {
                Some(s) => s
                    .parse::<u32>()
                    .map_err(|_| format!("Non-numeric segment '{s}' in version '{raw}'")),
                None => Ok(0),
            }
        };
        Ok(Self {
            major: parse_seg(0)?,
            minor: parse_seg(1)?,
            patch: parse_seg(2)?,
            build: parse_seg(3)?,
        })
    }

    pub fn within_inclusive_range(&self, min_v: &Self, max_v: &Self) -> bool {
        self >= min_v && self <= max_v
    }
}

impl std::fmt::Display for EcosystemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.build > 0 {
            write!(f, "{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Pin and compatibility window for a single foundational ecosystem component (e.g. `qualiadb-core`,
/// `vibescript-engine`, `etalon-gpu`, `aether-ui`, `webizen-synth-morphisms`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentVersionPin {
    pub component_id: String,
    pub display_name: String,
    /// The exact version the `.qap` was authored and verified against.
    pub pinned_designed_version: String,
    /// Minimum compatible version (inclusive).
    pub min_compatible_version: String,
    /// Maximum compatible version for minor/patch updates before a breaking epoch boundary (inclusive).
    pub max_compatible_version: String,
    /// Canonical wire/ABI epoch token (e.g. `"NQuin48-Q42-v2"`, `"HOLO-0x0306-10D-v2"`).
    pub abi_or_wire_epoch: String,
    /// Optional git revision or specification digest pin.
    pub source_rev_or_digest_pin: Option<String>,
    /// Human-readable explanation of what this component supplies and what changes across epochs.
    pub rationale_and_migration_hint: String,
}

/// Explicit API surface contract for a host capability (`capability.invoke`) required by a `.qap`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySurfaceContract {
    pub capability_id: String,
    pub owning_component_id: String,
    pub introduced_in_version: String,
    pub expected_input_fields: Vec<String>,
    pub expected_output_fields: Vec<String>,
    pub surface_signature_hash_hex: String,
    pub breaking_change_guidance: String,
}

impl CapabilitySurfaceContract {
    pub fn new(
        capability_id: impl Into<String>,
        owning_component_id: impl Into<String>,
        introduced_in_version: impl Into<String>,
        expected_input_fields: Vec<&str>,
        expected_output_fields: Vec<&str>,
        breaking_change_guidance: impl Into<String>,
    ) -> Self {
        let cap_id = capability_id.into();
        let owner = owning_component_id.into();
        let intro = introduced_in_version.into();
        let inputs: Vec<String> = expected_input_fields.into_iter().map(|s| s.to_string()).collect();
        let outputs: Vec<String> = expected_output_fields.into_iter().map(|s| s.to_string()).collect();
        let sig_hash = Self::compute_surface_hash(&cap_id, &inputs, &outputs);
        Self {
            capability_id: cap_id,
            owning_component_id: owner,
            introduced_in_version: intro,
            expected_input_fields: inputs,
            expected_output_fields: outputs,
            surface_signature_hash_hex: sig_hash,
            breaking_change_guidance: breaking_change_guidance.into(),
        }
    }

    pub fn compute_surface_hash(
        capability_id: &str,
        inputs: &[String],
        outputs: &[String],
    ) -> String {
        let material = format!(
            "QAP_CAP_SURFACE_V1|id={}|in={}|out={}",
            capability_id,
            inputs.join(","),
            outputs.join(",")
        );
        hex_encode(&compute_entry_digest32(material.as_bytes()))
    }
}

/// Complete Version-Pinning & API-Surface Compatibility Matrix embedded inside every `.qap`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QapEnvironmentCompatibilityMatrix {
    pub qap_id: String,
    pub qap_version: String,
    pub qapc_compiler_version_pin: String,
    pub allow_minor_and_patch_host_updates: bool,
    pub component_pins: Vec<ComponentVersionPin>,
    pub capability_surface_contracts: Vec<CapabilitySurfaceContract>,
}

impl QapEnvironmentCompatibilityMatrix {
    /// Builds the default canonical compatibility matrix for a `.qap` package, pinning the current
    /// `0.0.41` ecosystem components with minor/patch tolerance (`0.0.40` ..= `0.0.49.99`) and
    /// deriving explicit API surface contracts for the `.qap`'s required capabilities.
    pub fn default_for_qap(
        qap_id: &str,
        qap_version: &str,
        required_capabilities: &[String],
    ) -> Self {
        let component_pins = vec![
            ComponentVersionPin {
                component_id: "qualiadb-core".to_string(),
                display_name: "QualiaDB Core (.q42 NQuin48, 42 MiB Prolog Sentinel & Bitemporal Graph)".to_string(),
                pinned_designed_version: "0.0.41".to_string(),
                min_compatible_version: "0.0.40".to_string(),
                max_compatible_version: "0.0.49.99".to_string(),
                abi_or_wire_epoch: "NQuin48-Q42-v2".to_string(),
                source_rev_or_digest_pin: Some("branch:0.0.41".to_string()),
                rationale_and_migration_hint: "Requires 48-byte NQuin48 6-column quad layout and FixedArena <= 42 MiB Sentinel.".to_string(),
            },
            ComponentVersionPin {
                component_id: "vibescript-engine".to_string(),
                display_name: "QualiaDB VibeScript Engine (crates/vibe & vibescript-abi)".to_string(),
                pinned_designed_version: "0.0.41".to_string(),
                min_compatible_version: "0.0.40".to_string(),
                max_compatible_version: "0.2.99.99".to_string(),
                abi_or_wire_epoch: "VibeScript-0.1-0.2-Compatible-ABI".to_string(),
                source_rev_or_digest_pin: Some("spec:vibescript-0.2-compatible-0.1".to_string()),
                rationale_and_migration_hint: "Retains pinned 0.1 `requires [ capability(...) ];` and `effect fn` grammar while supporting 0.2 host effect extensions.".to_string(),
            },
            ComponentVersionPin {
                component_id: "etalon-gpu".to_string(),
                display_name: "Etalon Pure-Rust GPU & Holographic 10D v2 Compute Stack".to_string(),
                pinned_designed_version: "0.0.2".to_string(),
                min_compatible_version: "0.0.2".to_string(),
                max_compatible_version: "0.0.9.99".to_string(),
                abi_or_wire_epoch: "HOLO-0x0306-10D-v2".to_string(),
                source_rev_or_digest_pin: Some("rev:d4a6af1ad689c826".to_string()),
                rationale_and_migration_hint: "Requires FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE (0x0306) and Etalon Naga WGSL validation.".to_string(),
            },
            ComponentVersionPin {
                component_id: "aether-ui".to_string(),
                display_name: "Aether Native GPU (Blitz + Vello + Refract) & Mobile RSX Toolkit".to_string(),
                pinned_designed_version: "0.0.41".to_string(),
                min_compatible_version: "0.0.40".to_string(),
                max_compatible_version: "0.0.49.99".to_string(),
                abi_or_wire_epoch: "Aether-RSX-Native-B1a".to_string(),
                source_rev_or_digest_pin: Some("branch:0.0.41".to_string()),
                rationale_and_migration_hint: "Requires Aether native window lifecycle and zero-heap Refract lens signals.".to_string(),
            },
            ComponentVersionPin {
                component_id: "webizen-synth-morphisms".to_string(),
                display_name: "Webizen AI Synth & Hypermedia-Production 400-Morphism Catalog".to_string(),
                pinned_designed_version: "0.0.41.20".to_string(),
                min_compatible_version: "0.0.41.0".to_string(),
                max_compatible_version: "0.0.45.99".to_string(),
                abi_or_wire_epoch: "Synth-400-Morphisms-1200-NQuins".to_string(),
                source_rev_or_digest_pin: Some("catalog:400-morphisms-58-categories".to_string()),
                rationale_and_migration_hint: "Requires aligned 400-morphism catalog (150 base + 250 expansion).".to_string(),
            },
        ];

        let mut contracts = Vec::new();
        for cap in required_capabilities {
            if cap == "capability.invoke" {
                continue;
            }
            contracts.push(known_capability_surface_contract(cap));
        }

        Self {
            qap_id: qap_id.to_string(),
            qap_version: qap_version.to_string(),
            qapc_compiler_version_pin: "0.0.41".to_string(),
            allow_minor_and_patch_host_updates: true,
            component_pins,
            capability_surface_contracts: contracts,
        }
    }
}

/// Canonical registry of expected input/output field schemas for ecosystem capabilities so the
/// compiler can detect when a host environment's API surface changes.
pub fn known_capability_surface_contract(capability_id: &str) -> CapabilitySurfaceContract {
    match capability_id {
        "Biophysical.estimate_rppg_hrv" | "Biophysical.estimate_chrom_rppg" => {
            CapabilitySurfaceContract::new(
                capability_id,
                "qualiadb-core",
                "0.0.41",
                vec!["fps", "target_bpm"],
                vec!["bpm_x10", "rmssd_ms", "liveness_pad_ok"],
                "If rPPG output fields change (e.g. bpm_x10 -> heart_rate_bpm), update vibe/main.vibe telemetry projection.",
            )
        }
        "Pharmacokinetics.solve_rk4" => CapabilitySurfaceContract::new(
            capability_id,
            "qualiadb-core",
            "0.0.41",
            vec!["med"],
            vec!["steady_state"],
            "Expects 1-compartment oral absorption + elimination RK4 solver returning steady_state window status.",
        ),
        "SanctuaryStaging.stage_import" | "Sanctuary.stage_uncategorized_import" => {
            CapabilitySurfaceContract::new(
                capability_id,
                "qualiadb-core",
                "0.0.41",
                vec!["source"],
                vec!["staging_uri", "isolation"],
                "Must stage uncategorized imports into sanctuary://intake/uncategorized with QuarantinedUncategorized isolation.",
            )
        }
        "QdpDomainMail.resolve_delivery" | "QdpMail.resolve_domain_mailbox" => {
            CapabilitySurfaceContract::new(
                capability_id,
                "qualiadb-core",
                "0.0.41",
                vec!["domain"],
                vec!["qdp_txt"],
                "Expects _qdp.<domain> DNS TXT and /.well-known/QDP resolution.",
            )
        }
        "CoopLedger.evaluate_transition_readiness" => CapabilitySurfaceContract::new(
            capability_id,
            "qualiadb-core",
            "0.0.41",
            vec!["project"],
            vec!["pool_funded_s_cents", "target_t_cents", "invariant"],
            "Enforces Pool Funded != Contributor Paid invariant across 9-stage contribution lifecycle.",
        ),
        "CoopPlan.schedule_cpm" | "CoopPlan.calculate_cpm" => CapabilitySurfaceContract::new(
            capability_id,
            "qualiadb-core",
            "0.0.41",
            vec!["project"],
            vec!["project_finish_min", "critical_path_tasks"],
            "Expects PDM/CPM forward/backward pass schedule metrics.",
        ),
        "CoopChatGraph.verify_delegated_agent" => CapabilitySurfaceContract::new(
            capability_id,
            "qualiadb-core",
            "0.0.41",
            vec!["session"],
            vec!["standing", "independent_legal_entity"],
            "Enforces DelegatedInstrumentOfNaturalPerson standing (independent_legal_entity = false).",
        ),
        "GraphDatabase.lexicon_manifest" => CapabilitySurfaceContract::new(
            capability_id,
            "qualiadb-core",
            "0.0.41",
            vec!["version"],
            vec!["status"],
            "Expects Q42LEX WordNet 3.1 + OMW lexicon manifest descriptor.",
        ),
        "WgslForge.generate" => CapabilitySurfaceContract::new(
            capability_id,
            "etalon-gpu",
            "0.0.2",
            vec!["shader_profile"],
            vec!["status"],
            "Expects Etalon Naga WGSL shader generator.",
        ),
        other => CapabilitySurfaceContract::new(
            other,
            "qualiadb-core",
            "0.0.41",
            vec![],
            vec!["status"],
            "Standard QualiaDB capability.invoke contract.",
        ),
    }
}

// ============================================================================
// HOST ENVIRONMENT PROFILE & COMPATIBILITY VERIFICATION ENGINE
// ============================================================================

/// Installed component entry in a user's runtime or build host environment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledHostComponent {
    pub component_id: String,
    pub installed_version: String,
    pub abi_or_wire_epoch: String,
}

/// Available capability surface in a user's runtime or build host environment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailableHostCapabilitySurface {
    pub capability_id: String,
    pub accepted_input_fields: Vec<String>,
    pub provided_output_fields: Vec<String>,
}

/// Complete description of a user's host environment against which a `.qap` is compiled or loaded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostEnvironmentProfile {
    pub environment_label: String,
    pub installed_components: Vec<InstalledHostComponent>,
    pub available_capabilities: Vec<AvailableHostCapabilitySurface>,
    pub removed_capability_ids: Vec<String>,
    pub include_standard_catalog_capabilities: bool,
}

impl HostEnvironmentProfile {
    /// Returns the baseline `2026-11` (`v0.0.41`) host environment matching the exact pinned design target.
    pub fn baseline_pinned_2026_11() -> Self {
        Self {
            environment_label: "Webizen Native Desktop/Mobile Host v0.0.41 (Exact Pinned Baseline)".to_string(),
            installed_components: vec![
                InstalledHostComponent {
                    component_id: "qualiadb-core".to_string(),
                    installed_version: "0.0.41".to_string(),
                    abi_or_wire_epoch: "NQuin48-Q42-v2".to_string(),
                },
                InstalledHostComponent {
                    component_id: "vibescript-engine".to_string(),
                    installed_version: "0.0.41".to_string(),
                    abi_or_wire_epoch: "VibeScript-0.1-0.2-Compatible-ABI".to_string(),
                },
                InstalledHostComponent {
                    component_id: "etalon-gpu".to_string(),
                    installed_version: "0.0.2".to_string(),
                    abi_or_wire_epoch: "HOLO-0x0306-10D-v2".to_string(),
                },
                InstalledHostComponent {
                    component_id: "aether-ui".to_string(),
                    installed_version: "0.0.41".to_string(),
                    abi_or_wire_epoch: "Aether-RSX-Native-B1a".to_string(),
                },
                InstalledHostComponent {
                    component_id: "webizen-synth-morphisms".to_string(),
                    installed_version: "0.0.41.20".to_string(),
                    abi_or_wire_epoch: "Synth-400-Morphisms-1200-NQuins".to_string(),
                },
            ],
            available_capabilities: [
                "Biophysical.estimate_rppg_hrv",
                "Biophysical.estimate_chrom_rppg",
                "Pharmacokinetics.solve_rk4",
                "SanctuaryStaging.stage_import",
                "Sanctuary.stage_uncategorized_import",
                "QdpDomainMail.resolve_delivery",
                "QdpMail.resolve_domain_mailbox",
                "CoopLedger.evaluate_transition_readiness",
                "CoopPlan.schedule_cpm",
                "CoopPlan.calculate_cpm",
                "CoopChatGraph.verify_delegated_agent",
                "MultiRailWallet.reconcile_provider_credit",
                "GraphDatabase.lexicon_manifest",
                "GraphDatabase.sparql",
                "WgslForge.generate",
                "ClinicalRisk.framingham",
            ]
            .into_iter()
            .map(|id| {
                let c = known_capability_surface_contract(id);
                AvailableHostCapabilitySurface {
                    capability_id: c.capability_id,
                    accepted_input_fields: c.expected_input_fields,
                    provided_output_fields: c.expected_output_fields,
                }
            })
            .collect(),
            removed_capability_ids: Vec::new(),
            include_standard_catalog_capabilities: true,
        }
    }

    /// Simulates a user host environment that has received minor/patch updates
    /// (e.g. `qualiadb-core 0.0.43.5`, `etalon-gpu 0.0.4`, `webizen-synth-morphisms 0.0.42.10`)
    /// without breaking the ABI epochs or capability field surfaces.
    pub fn minor_updated_host_environment() -> Self {
        let mut env = Self::baseline_pinned_2026_11();
        env.environment_label =
            "User Host Environment with Minor/Patch Updates (QualiaDB 0.0.43.5, Etalon 0.0.4, Synth 0.0.42.10)"
                .to_string();
        for comp in &mut env.installed_components {
            match comp.component_id.as_str() {
                "qualiadb-core" | "aether-ui" => comp.installed_version = "0.0.43.5".to_string(),
                "vibescript-engine" => comp.installed_version = "0.1.2".to_string(),
                "etalon-gpu" => comp.installed_version = "0.0.4".to_string(),
                "webizen-synth-morphisms" => comp.installed_version = "0.0.42.10".to_string(),
                _ => {}
            }
        }
        env
    }
}

/// Granular compatibility status for a component pin or capability contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    /// Installed host matches the exact `pinned_designed_version` and API surface hash.
    ExactPinnedMatch,
    /// Installed host has a minor or patch update within `[min_compatible ..= max_compatible]`
    /// and preserves the required ABI epoch and capability API surface fields.
    /// Compilation and execution succeed without disruption.
    CompatibleMinorOrPatchDrift,
    /// Installed host version is outside `[min_compatible ..= max_compatible]` or has a different ABI epoch.
    IncompatibleVersionOrAbiEpoch,
    /// A required capability is missing or its input/output field surface has changed.
    IncompatibleApiSurfaceChange,
}

impl CompatibilityStatus {
    pub const fn can_compile_and_run(self) -> bool {
        matches!(
            self,
            Self::ExactPinnedMatch | Self::CompatibleMinorOrPatchDrift
        )
    }
}

/// Diagnostic finding for a single component or capability surface check.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityFinding {
    pub subject_id: String,
    pub status: CompatibilityStatus,
    pub designed_for: String,
    pub host_actual: String,
    pub detail_and_remedy: String,
}

/// Complete Compatibility & API Surface Report comparing a `.qap`'s `QapEnvironmentCompatibilityMatrix`
/// against a user's `HostEnvironmentProfile`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCompatibilityReport {
    pub qap_id: String,
    pub host_environment_label: String,
    pub overall_status: CompatibilityStatus,
    pub can_compile_and_activate: bool,
    pub findings: Vec<CompatibilityFinding>,
    pub compatibility_receipt_quin: NQuin48,
}

/// Evaluates whether a `.qap` package (`matrix`) can compile and run cleanly against `host_env`,
/// distinguishing benign minor/patch drift from breaking ABI epoch or capability surface changes.
pub fn evaluate_qap_against_host_environment(
    matrix: &QapEnvironmentCompatibilityMatrix,
    host_env: &HostEnvironmentProfile,
) -> EnvironmentCompatibilityReport {
    let mut findings = Vec::new();
    let mut worst_status = CompatibilityStatus::ExactPinnedMatch;

    let elevate = |current: &mut CompatibilityStatus, next: CompatibilityStatus| {
        use CompatibilityStatus::*;
        let rank = |s: CompatibilityStatus| match s {
            ExactPinnedMatch => 0,
            CompatibleMinorOrPatchDrift => 1,
            IncompatibleVersionOrAbiEpoch => 2,
            IncompatibleApiSurfaceChange => 3,
        };
        if rank(next) > rank(*current) {
            *current = next;
        }
    };

    // 1. Check all component version pins & ABI epochs
    for pin in &matrix.component_pins {
        let Some(installed) = host_env
            .installed_components
            .iter()
            .find(|c| c.component_id == pin.component_id)
        else {
            elevate(
                &mut worst_status,
                CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("component:{}", pin.component_id),
                status: CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
                designed_for: format!(
                    "{} ({})",
                    pin.pinned_designed_version, pin.abi_or_wire_epoch
                ),
                host_actual: "NOT_INSTALLED".to_string(),
                detail_and_remedy: format!(
                    "Missing required component '{}'. {}",
                    pin.display_name, pin.rationale_and_migration_hint
                ),
            });
            continue;
        };

        if installed.abi_or_wire_epoch != pin.abi_or_wire_epoch {
            elevate(
                &mut worst_status,
                CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("component:{}", pin.component_id),
                status: CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
                designed_for: format!(
                    "{} (ABI: {})",
                    pin.pinned_designed_version, pin.abi_or_wire_epoch
                ),
                host_actual: format!(
                    "{} (ABI: {})",
                    installed.installed_version, installed.abi_or_wire_epoch
                ),
                detail_and_remedy: format!(
                    "Breaking ABI/wire epoch change on '{}': package was designed for '{}', host has '{}'. {}",
                    pin.component_id,
                    pin.abi_or_wire_epoch,
                    installed.abi_or_wire_epoch,
                    pin.rationale_and_migration_hint
                ),
            });
            continue;
        }

        let pinned_v = EcosystemVersion::parse(&pin.pinned_designed_version)
            .unwrap_or(EcosystemVersion::new(0, 0, 41, 0));
        let min_v = EcosystemVersion::parse(&pin.min_compatible_version).unwrap_or(pinned_v);
        let max_v = EcosystemVersion::parse(&pin.max_compatible_version).unwrap_or(pinned_v);
        let actual_v = EcosystemVersion::parse(&installed.installed_version)
            .unwrap_or(EcosystemVersion::new(0, 0, 0, 0));

        if actual_v == pinned_v {
            findings.push(CompatibilityFinding {
                subject_id: format!("component:{}", pin.component_id),
                status: CompatibilityStatus::ExactPinnedMatch,
                designed_for: pin.pinned_designed_version.clone(),
                host_actual: installed.installed_version.clone(),
                detail_and_remedy: format!(
                    "Exact pinned version match ({}) and ABI epoch '{}'.",
                    pin.pinned_designed_version, pin.abi_or_wire_epoch
                ),
            });
        } else if matrix.allow_minor_and_patch_host_updates
            && actual_v.within_inclusive_range(&min_v, &max_v)
        {
            elevate(
                &mut worst_status,
                CompatibilityStatus::CompatibleMinorOrPatchDrift,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("component:{}", pin.component_id),
                status: CompatibilityStatus::CompatibleMinorOrPatchDrift,
                designed_for: format!(
                    "pinned={} (compatible {}..={})",
                    pin.pinned_designed_version,
                    pin.min_compatible_version,
                    pin.max_compatible_version
                ),
                host_actual: installed.installed_version.clone(),
                detail_and_remedy: format!(
                    "Compatible minor/patch update on '{}': designed for {}, host is {} (within {}..={}, ABI '{}' unchanged).",
                    pin.component_id,
                    pin.pinned_designed_version,
                    installed.installed_version,
                    pin.min_compatible_version,
                    pin.max_compatible_version,
                    pin.abi_or_wire_epoch
                ),
            });
        } else {
            elevate(
                &mut worst_status,
                CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("component:{}", pin.component_id),
                status: CompatibilityStatus::IncompatibleVersionOrAbiEpoch,
                designed_for: format!(
                    "pinned={} (compatible {}..={})",
                    pin.pinned_designed_version,
                    pin.min_compatible_version,
                    pin.max_compatible_version
                ),
                host_actual: installed.installed_version.clone(),
                detail_and_remedy: format!(
                    "Host version {} for '{}' is outside compatible window [{} ..= {}]. {}",
                    installed.installed_version,
                    pin.component_id,
                    pin.min_compatible_version,
                    pin.max_compatible_version,
                    pin.rationale_and_migration_hint
                ),
            });
        }
    }

    // 2. Check all required capability API surface contracts (input & output fields)
    for contract in &matrix.capability_surface_contracts {
        let resolved_host_cap: Option<AvailableHostCapabilitySurface> = if host_env
            .removed_capability_ids
            .contains(&contract.capability_id)
        {
            None
        } else if let Some(explicit) = host_env
            .available_capabilities
            .iter()
            .find(|c| c.capability_id == contract.capability_id)
        {
            Some(explicit.clone())
        } else if host_env.include_standard_catalog_capabilities {
            let std_c = known_capability_surface_contract(&contract.capability_id);
            Some(AvailableHostCapabilitySurface {
                capability_id: std_c.capability_id,
                accepted_input_fields: std_c.expected_input_fields,
                provided_output_fields: std_c.expected_output_fields,
            })
        } else {
            None
        };

        let Some(host_cap) = resolved_host_cap else {
            elevate(
                &mut worst_status,
                CompatibilityStatus::IncompatibleApiSurfaceChange,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("capability:{}", contract.capability_id),
                status: CompatibilityStatus::IncompatibleApiSurfaceChange,
                designed_for: format!(
                    "{} (introduced in {} {}, inputs={:?}, outputs={:?})",
                    contract.capability_id,
                    contract.owning_component_id,
                    contract.introduced_in_version,
                    contract.expected_input_fields,
                    contract.expected_output_fields
                ),
                host_actual: "CAPABILITY_MISSING_ON_HOST".to_string(),
                detail_and_remedy: format!(
                    "Required capability '{}' (from '{}') is missing in host environment. {}",
                    contract.capability_id,
                    contract.owning_component_id,
                    contract.breaking_change_guidance
                ),
            });
            continue;
        };

        let missing_inputs: Vec<String> = contract
            .expected_input_fields
            .iter()
            .filter(|f| !host_cap.accepted_input_fields.contains(f))
            .cloned()
            .collect();
        let missing_outputs: Vec<String> = contract
            .expected_output_fields
            .iter()
            .filter(|f| !host_cap.provided_output_fields.contains(f))
            .cloned()
            .collect();

        if !missing_inputs.is_empty() || !missing_outputs.is_empty() {
            elevate(
                &mut worst_status,
                CompatibilityStatus::IncompatibleApiSurfaceChange,
            );
            findings.push(CompatibilityFinding {
                subject_id: format!("capability:{}", contract.capability_id),
                status: CompatibilityStatus::IncompatibleApiSurfaceChange,
                designed_for: format!(
                    "in={:?}, out={:?} (sig={})",
                    contract.expected_input_fields,
                    contract.expected_output_fields,
                    &contract.surface_signature_hash_hex[..12]
                ),
                host_actual: format!(
                    "in={:?}, out={:?}",
                    host_cap.accepted_input_fields, host_cap.provided_output_fields
                ),
                detail_and_remedy: format!(
                    "API Surface Change detected on '{}' (designed for {} {}): missing_inputs={:?}, missing_outputs={:?}. Guidance: {}",
                    contract.capability_id,
                    contract.owning_component_id,
                    contract.introduced_in_version,
                    missing_inputs,
                    missing_outputs,
                    contract.breaking_change_guidance
                ),
            });
        } else {
            findings.push(CompatibilityFinding {
                subject_id: format!("capability:{}", contract.capability_id),
                status: CompatibilityStatus::ExactPinnedMatch,
                designed_for: format!(
                    "in={:?}, out={:?}",
                    contract.expected_input_fields, contract.expected_output_fields
                ),
                host_actual: format!(
                    "in={:?}, out={:?}",
                    host_cap.accepted_input_fields, host_cap.provided_output_fields
                ),
                detail_and_remedy: format!(
                    "Capability '{}' API surface verified (all required input/output fields present).",
                    contract.capability_id
                ),
            });
        }
    }

    let can_compile = worst_status.can_compile_and_run();
    let quin = NQuin48::new(
        q_hash_u64(&matrix.qap_id),
        q_hash_u64("qap:evaluatedHostCompatibility"),
        if can_compile { 1 } else { 0 },
        q_hash_u64(&host_env.environment_label),
        findings.len() as u64,
    );

    EnvironmentCompatibilityReport {
        qap_id: matrix.qap_id.clone(),
        host_environment_label: host_env.environment_label.clone(),
        overall_status: worst_status,
        can_compile_and_activate: can_compile,
        findings,
        compatibility_receipt_quin: quin,
    }
}
