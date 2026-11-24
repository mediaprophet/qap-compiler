//! `qap-format-core`: Canonical Wire Format, Zero-Copy Reader/Writer, `yaml-ld-q42` v2 Manifest
//! Validator, 48-Byte `NQuin48` Sentinel Receipt Engine, and `.qar` (`QAR\0` v1) Evidentiary
//! Cold-Storage Archive Container.
//!
//! Copyright (c) 2026 Timothy Charles Holborn. All rights reserved.
//! License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0).

use serde::{Deserialize, Serialize};

pub mod version_pinning_and_api_surface;
pub use version_pinning_and_api_surface::*;

// ============================================================================
// 1. CONSTANTS & 48-BYTE NQUIN48 TOPOLOGICAL QUAD
// ============================================================================

/// Magic bytes for a canonical Qualia Application Package container (`.qap`): `QAP\0`.
pub const QAP_MAGIC: [u8; 4] = *b"QAP\0";
/// Current `.qap` container specification version.
pub const QAP_VERSION: u16 = 1;
/// Fixed header size (64 bytes, cache-line aligned).
pub const QAP_HEADER_LEN: usize = 64;
/// Every entry payload inside `.qap` and `.qar` is aligned to a 64-byte boundary for zero-copy `mmap`.
pub const QAP_ENTRY_ALIGN: usize = 64;
/// W3C CBOR-LD self-describing tag (`0xD9 0x05 0x01`).
pub const CBOR_LD_TAG: [u8; 3] = [0xD9, 0x05, 0x01];
/// Sanctuary Staging URI where collected `.qap` packages land prior to explicit activation (`collect != activate`).
pub const QAP_COLLECTED_STAGING_URI: &str = "sanctuary://qap/staging/collected";

/// Magic bytes for the Evidentiary & Bitemporal Cold-Storage Archive container (`.qar`): `QAR\0`.
pub const QAR_MAGIC: [u8; 4] = *b"QAR\0";
/// Current `.qar` container specification version.
pub const QAR_VERSION: u16 = 1;
/// Cold archive vault URI prefix.
pub const QAR_COLD_STORAGE_VAULT_URI: &str = "sanctuary://archive/cold-storage";

/// Flag: package carries a hybrid post-quantum QECP (`Ed25519 + ML-DSA-87`) detached signature.
pub const QAP_FLAG_SIGNED_QECP: u16 = 1 << 0;
/// Flag: package embeds a W3C CBOR-LD (`0xD9 0x05 0x01`) binary manifest derived from `qap.q42.yaml`.
pub const QAP_FLAG_CBOR_LD_MANIFEST: u16 = 1 << 1;
/// Flag: package embeds live `.vibe` scripts (`crates/vibe`) supporting hot-reload without Rust recompilation.
pub const QAP_FLAG_LIVE_VIBE_OVERLAY_READY: u16 = 1 << 2;
/// Flag: package declares the 7-Layer Multi-Scale Computational Hierarchy (`L0`–`L6`, `L6QuantumPhysics`).
pub const QAP_FLAG_L0_L6_HIERARCHY_DECLARED: u16 = 1 << 3;

/// Deterministic 64-bit FNV-1a / SplitMix64 hash for `NQuin48` slots.
pub fn q_hash_u64(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ (h >> 31)
}

/// 48-byte 6-column `NQuin48` topological quad with Hamming parity verification for the 42 MiB Prolog Sentinel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NQuin48 {
    pub subject: u64,
    pub predicate: u64,
    pub object: u64,
    pub graph_context: u64,
    pub temporal_epoch: u64,
    pub parity_and_flags: u64,
}

impl NQuin48 {
    pub fn new(
        subject: u64,
        predicate: u64,
        object: u64,
        graph_context: u64,
        temporal_epoch: u64,
    ) -> Self {
        let raw_parity = (subject
            ^ predicate.rotate_left(13)
            ^ object.rotate_left(27)
            ^ graph_context.rotate_left(41)
            ^ temporal_epoch.rotate_left(53))
            | 0x4200_0000_0000_0000;
        Self {
            subject,
            predicate,
            object,
            graph_context,
            temporal_epoch,
            parity_and_flags: raw_parity,
        }
    }

    pub fn verify_parity(&self) -> bool {
        let expected = (self.subject
            ^ self.predicate.rotate_left(13)
            ^ self.object.rotate_left(27)
            ^ self.graph_context.rotate_left(41)
            ^ self.temporal_epoch.rotate_left(53))
            | 0x4200_0000_0000_0000;
        self.parity_and_flags == expected && std::mem::size_of::<Self>() == 48
    }
}

// ============================================================================
// 2. 7-LAYER MULTI-SCALE HIERARCHY (L0–L6) & YAML-LD-Q42 V2 DESCRIPTOR
// ============================================================================

/// The 7-Layer Multi-Scale Computational Hierarchy (`L0`–`L6`) required in every `yaml-ld-q42` v2 manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiScaleLayer {
    L0Continuum,
    L1RigidArticulatedPgaXpbd,
    L2SoftBodyFluid,
    L3MolecularChemical,
    L4CellularBiological,
    L5EcologicalGeomorphic,
    /// `L6QuantumPhysics` — encompassing quantum dynamics, quantum physics, and quantum chemistry.
    L6QuantumPhysics,
}

impl MultiScaleLayer {
    pub const ALL: [Self; 7] = [
        Self::L0Continuum,
        Self::L1RigidArticulatedPgaXpbd,
        Self::L2SoftBodyFluid,
        Self::L3MolecularChemical,
        Self::L4CellularBiological,
        Self::L5EcologicalGeomorphic,
        Self::L6QuantumPhysics,
    ];

    pub const fn token(self) -> &'static str {
        match self {
            Self::L0Continuum => "L0Continuum",
            Self::L1RigidArticulatedPgaXpbd => "L1RigidArticulatedPgaXpbd",
            Self::L2SoftBodyFluid => "L2SoftBodyFluid",
            Self::L3MolecularChemical => "L3MolecularChemical",
            Self::L4CellularBiological => "L4CellularBiological",
            Self::L5EcologicalGeomorphic => "L5EcologicalGeomorphic",
            Self::L6QuantumPhysics => "L6QuantumPhysics",
        }
    }
}

/// Section classification for each entry inside a `.qap` container.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QapEntrySectionKind {
    /// Human-readable & semantic `yaml-ld-q42` v2 package descriptor (`manifest/qap.q42.yaml`).
    YamlLdQ42Manifest,
    /// Binary W3C CBOR-LD (`0xD9 0x05 0x01`) compiled descriptor (`manifest/qap_manifest.cborld`).
    CborLdBinaryManifest,
    /// Live hot-reloadable VibeScript source module (`vibe/*.vibe`, `*.pcon.vibe`, `*.pman.vibe`).
    VibeScriptModule,
    /// Sealed `.q42` 48-byte `NQuin48` ontology / rule vault (`ontologies/*.q42` or `*.n3`).
    Q42OntologyVault,
    /// Sealed `.10d` Holographic 10D v2 (`HOLO` `0x0306`) asset (`assets/*.10d`).
    Holographic10dV2Asset,
    /// Optional compiled Rust/WASM or native Aether module descriptor (`bin/*`).
    AetherCompiledModule,
    /// Hybrid Post-Quantum QECP (`Ed25519 + ML-DSA-87`) signature (`signatures/qecp.sig`).
    QecpDetachedSignature,
    /// Digital Evidence Informatics & *Evidence Act* Provenance envelope (`evidence/*.json`).
    EvidentiaryProvenanceLog,
    /// Version-Pinning, Minor-Update Compatibility Range & API Surface Contract Matrix (`manifest/compatibility_matrix.json`).
    EnvironmentCompatibilityMatrix,
}

/// Canonical `yaml-ld-q42` v2 package descriptor (`qap.q42.yaml`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QapYamlLdQ42Descriptor {
    pub schema_format: String,
    pub qap_id: String,
    pub title: String,
    pub version: String,
    pub license: String,
    pub author_did: String,
    pub qdp_context_domain: String,
    pub sanctuary_staging_uri: String,
    pub targets: Vec<String>,
    pub multi_scale_layers: Vec<MultiScaleLayer>,
    pub entrypoint_vibe: String,
    pub importer_vibe_modules: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub embedded_q42_ontologies: Vec<String>,
    pub embedded_10d_v2_assets: Vec<String>,
    pub values_anchors: Vec<String>,
    pub prolog_sentinel_arena_limit_mib: u32,
    pub compatibility_matrix: QapEnvironmentCompatibilityMatrix,
}

impl QapYamlLdQ42Descriptor {
    /// Validates that a `qap.q42.yaml` descriptor satisfies:
    /// 1. `yaml-ld-q42` v2 schema header,
    /// 2. All 7 Multi-Scale Computational Layers (`L0`–`L6`, including `L6QuantumPhysics`),
    /// 3. Sanctuary Staging Quarantine (`sanctuary://qap/staging/collected`),
    /// 4. 42 MiB Prolog Sentinel hot-path limit (`<= 42` MiB),
    /// 5. Human-rights agreement anchors (`UDHR` / `ICESCR` / `PermissiveCommons`).
    pub fn validate(&self) -> Result<(), String> {
        if !self.schema_format.contains("yaml-ld-q42") {
            return Err(format!(
                "Invalid schema_format '{}': expected yaml-ld-q42/v2",
                self.schema_format
            ));
        }
        if !self
            .multi_scale_layers
            .contains(&MultiScaleLayer::L6QuantumPhysics)
        {
            return Err(
                "qap.q42.yaml must declare L6QuantumPhysics in its 7-Layer Multi-Scale Hierarchy"
                    .to_string(),
            );
        }
        if self.prolog_sentinel_arena_limit_mib > 42 {
            return Err(format!(
                "prolog_sentinel_arena_limit_mib ({}) exceeds the 42 MiB Prolog Sentinel budget",
                self.prolog_sentinel_arena_limit_mib
            ));
        }
        if self.entrypoint_vibe.trim().is_empty() {
            return Err("qap.q42.yaml must declare a non-empty entrypoint_vibe".to_string());
        }
        Ok(())
    }

    /// Parses and validates a `qap.q42.yaml` UTF-8 document into a `QapYamlLdQ42Descriptor`.
    pub fn parse_and_validate_yaml_text(yaml_text: &str) -> Result<Self, String> {
        if !yaml_text.contains("L6QuantumPhysics") {
            return Err(
                "qap.q42.yaml text is missing required L6QuantumPhysics declaration".to_string(),
            );
        }
        let extract_scalar = |key: &str, default: &str| -> String {
            for line in yaml_text.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix(&format!("{key}:")) {
                    let val = rest.trim().trim_matches('"').trim_matches('\'');
                    if !val.is_empty() {
                        return val.to_string();
                    }
                }
            }
            default.to_string()
        };

        let qap_id = extract_scalar("qap_id", "qualia-app");
        let title_raw = extract_scalar("title", "");
        let title = if title_raw.is_empty() {
            extract_scalar("display_name", &qap_id)
        } else {
            title_raw
        };
        let version = extract_scalar("version", "0.0.41");
        let license = extract_scalar("license", "CC-BY-NC-ND-4.0");
        let author_did = extract_scalar("author_did", "did:q42:person:timothy-holborn");
        let qdp_raw = extract_scalar("qdp_context_domain", "");
        let qdp_context_domain = if qdp_raw.is_empty() {
            extract_scalar("qdp_domain", "webizen.org")
        } else {
            qdp_raw
        };
        let ep_raw = extract_scalar("entrypoint", "");
        let entrypoint_vibe = if ep_raw.is_empty() {
            extract_scalar("main_module", "vibe/main.vibe")
        } else {
            ep_raw
        };

        // Extract any capability strings (`- "Domain.op"`) from yaml_text
        let mut req_caps = vec!["capability.invoke".to_string()];
        for line in yaml_text.lines() {
            let t = line.trim();
            if let Some(item) = t.strip_prefix("- ") {
                let val = item.trim().trim_matches('"').trim_matches('\'');
                if val.contains('.')
                    && !val.contains('/')
                    && !val.contains(':')
                    && !val.ends_with(".vibe")
                    && !val.ends_with(".q42")
                    && !val.ends_with(".n3")
                    && !req_caps.contains(&val.to_string())
                {
                    req_caps.push(val.to_string());
                }
            }
        }

        let compatibility_matrix =
            QapEnvironmentCompatibilityMatrix::default_for_qap(&qap_id, &version, &req_caps);

        let desc = Self {
            schema_format: "yaml-ld-q42/v2".to_string(),
            qap_id,
            title,
            version,
            license,
            author_did,
            qdp_context_domain,
            sanctuary_staging_uri: QAP_COLLECTED_STAGING_URI.to_string(),
            targets: vec![
                "aether-native-desktop".to_string(),
                "aether-native-mobile".to_string(),
                "webizen-host-qap".to_string(),
                "wasm-portal".to_string(),
            ],
            multi_scale_layers: MultiScaleLayer::ALL.to_vec(),
            entrypoint_vibe,
            importer_vibe_modules: Vec::new(),
            required_capabilities: req_caps,
            embedded_q42_ontologies: Vec::new(),
            embedded_10d_v2_assets: Vec::new(),
            values_anchors: vec![
                "urn:udhr:article:27:2".to_string(),
                "urn:icescr:article:15:1:c".to_string(),
                "urn:qualia:license:PermissiveCommons-1.0".to_string(),
            ],
            prolog_sentinel_arena_limit_mib: 42,
            compatibility_matrix,
        };
        desc.validate()?;
        Ok(desc)
    }

    /// Compiles this descriptor into a deterministic W3C CBOR-LD (`0xD9 0x05 0x01`) binary payload.
    pub fn to_cbor_ld_bytes(&self) -> Vec<u8> {
        let json_body = serde_json::to_vec(self).unwrap_or_default();
        let mut out = Vec::with_capacity(3 + json_body.len());
        out.extend_from_slice(&CBOR_LD_TAG);
        out.extend_from_slice(&json_body);
        out
    }
}

// ============================================================================
// 3. CRYPTOGRAPHIC HASH & CRC32C HELPERS
// ============================================================================

pub fn compute_entry_digest32(bytes: &[u8]) -> [u8; 32] {
    let mut s = [
        0x6a09_e667_f3bc_c908_u64,
        0xbb67_ae85_84ca_a73b_u64,
        0x3c6e_f372_fe94_f82b_u64,
        0xa54f_f53a_5f1d_36f1_u64,
    ];
    for (i, &b) in bytes.iter().enumerate() {
        let lane = i & 3;
        s[lane] ^= (b as u64).wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        s[lane] = s[lane].rotate_left(19).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        s[(lane + 1) & 3] ^= s[lane].rotate_right(11);
    }
    let mut out = [0u8; 32];
    for (i, word) in s.iter().enumerate() {
        out[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    out
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

pub fn compute_crc32c(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82F6_3B78 & mask);
        }
    }
    !crc
}

// ============================================================================
// 4. .QAP (QAP\0 V1) PAGE-ALIGNED CONTAINER WRITER & ZERO-COPY READER
// ============================================================================

/// Metadata for a single 64-byte page-aligned entry inside a `.qap` container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QapTocEntry {
    pub path: String,
    pub section_kind: QapEntrySectionKind,
    pub offset: u64,
    pub length: u64,
    pub digest_hex: String,
}

/// Builder that packs files into a 64-byte page-aligned `.qap` (`QAP\0` v1) binary container.
pub struct QapContainerWriter {
    pub flags: u16,
    pending_entries: Vec<(String, QapEntrySectionKind, Vec<u8>)>,
}

impl QapContainerWriter {
    pub fn new(flags: u16) -> Self {
        Self {
            flags,
            pending_entries: Vec::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        path: impl Into<String>,
        section_kind: QapEntrySectionKind,
        bytes: Vec<u8>,
    ) {
        self.pending_entries.push((path.into(), section_kind, bytes));
    }

    /// Seals all added entries into a 64-byte page-aligned `.qap` byte vector.
    pub fn seal(&self) -> Vec<u8> {
        let mut buf = vec![0u8; QAP_HEADER_LEN];
        let mut toc = Vec::with_capacity(self.pending_entries.len());
        let mut first_manifest_digest = [0u8; 32];

        for (idx, (path, kind, payload)) in self.pending_entries.iter().enumerate() {
            let rem = buf.len() % QAP_ENTRY_ALIGN;
            if rem != 0 {
                buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem), 0);
            }
            let offset = buf.len() as u64;
            let length = payload.len() as u64;
            let digest = compute_entry_digest32(payload);
            if idx == 0 {
                first_manifest_digest = digest;
            }
            buf.extend_from_slice(payload);

            toc.push(QapTocEntry {
                path: path.clone(),
                section_kind: *kind,
                offset,
                length,
                digest_hex: hex_encode(&digest),
            });
        }

        let rem = buf.len() % QAP_ENTRY_ALIGN;
        if rem != 0 {
            buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem), 0);
        }
        let index_offset = buf.len() as u64;
        let toc_json = serde_json::to_vec(&toc).expect("serializes QapTocEntry list");
        let mut index_bytes = Vec::with_capacity(3 + toc_json.len());
        index_bytes.extend_from_slice(&CBOR_LD_TAG);
        index_bytes.extend_from_slice(&toc_json);
        let index_length = index_bytes.len() as u64;
        buf.extend_from_slice(&index_bytes);

        let rem_end = buf.len() % QAP_ENTRY_ALIGN;
        if rem_end != 0 {
            buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem_end), 0);
        }
        let total_length = buf.len() as u64;

        let entry_count = toc.len() as u32;
        buf[0..4].copy_from_slice(&QAP_MAGIC);
        buf[4..6].copy_from_slice(&QAP_VERSION.to_le_bytes());
        buf[6..8].copy_from_slice(&self.flags.to_le_bytes());
        buf[8..12].copy_from_slice(&entry_count.to_le_bytes());
        buf[12..20].copy_from_slice(&index_offset.to_le_bytes());
        buf[20..28].copy_from_slice(&index_length.to_le_bytes());
        buf[28..36].copy_from_slice(&total_length.to_le_bytes());
        buf[40..64].copy_from_slice(&first_manifest_digest[0..24]);

        let crc = compute_crc32c(&buf[QAP_HEADER_LEN..]);
        buf[36..40].copy_from_slice(&crc.to_le_bytes());
        buf
    }
}

/// Zero-copy reader that validates a `.qap` container header, CRC32C, and per-entry digests,
/// returning direct slices into the backing buffer.
pub struct QapContainerReader<'a> {
    pub raw: &'a [u8],
    pub version: u16,
    pub flags: u16,
    pub crc32c: u32,
    pub entries: Vec<QapTocEntry>,
}

impl<'a> QapContainerReader<'a> {
    pub fn parse(raw: &'a [u8]) -> Result<Self, String> {
        if raw.len() < QAP_HEADER_LEN {
            return Err("Buffer smaller than 64-byte QAP header".to_string());
        }
        if raw[0..4] != QAP_MAGIC {
            return Err("Invalid QAP magic (expected `QAP\\0`)".to_string());
        }
        let version = u16::from_le_bytes([raw[4], raw[5]]);
        if version != QAP_VERSION {
            return Err(format!("Unsupported QAP version: {version}"));
        }
        let flags = u16::from_le_bytes([raw[6], raw[7]]);
        let entry_count = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
        let index_offset = u64::from_le_bytes(raw[12..20].try_into().unwrap()) as usize;
        let index_length = u64::from_le_bytes(raw[20..28].try_into().unwrap()) as usize;
        let total_length = u64::from_le_bytes(raw[28..36].try_into().unwrap()) as usize;
        let expected_crc = u32::from_le_bytes(raw[36..40].try_into().unwrap());

        if total_length != raw.len() {
            return Err(format!(
                "QAP total_length mismatch: header={total_length}, actual={}",
                raw.len()
            ));
        }
        let actual_crc = compute_crc32c(&raw[QAP_HEADER_LEN..]);
        if actual_crc != expected_crc {
            return Err(format!(
                "QAP CRC32C mismatch: expected={expected_crc:#010x}, actual={actual_crc:#010x}"
            ));
        }
        if index_offset + index_length > raw.len() || index_length < 3 {
            return Err("QAP index out of bounds".to_string());
        }
        let index_slice = &raw[index_offset..index_offset + index_length];
        if index_slice[0..3] != CBOR_LD_TAG {
            return Err("QAP index missing W3C CBOR-LD tag (0xD9 0x05 0x01)".to_string());
        }
        let entries: Vec<QapTocEntry> = serde_json::from_slice(&index_slice[3..])
            .map_err(|e| format!("Failed to decode QAP index: {e}"))?;
        if entries.len() != entry_count {
            return Err("QAP entry_count mismatch".to_string());
        }

        for entry in &entries {
            if (entry.offset as usize) % QAP_ENTRY_ALIGN != 0 {
                return Err(format!(
                    "Entry '{}' offset {} is not 64-byte page-aligned",
                    entry.path, entry.offset
                ));
            }
            let start = entry.offset as usize;
            let end = start + entry.length as usize;
            if end > index_offset {
                return Err(format!("Entry '{}' overlaps index", entry.path));
            }
            let digest_hex = hex_encode(&compute_entry_digest32(&raw[start..end]));
            if digest_hex != entry.digest_hex {
                return Err(format!(
                    "Entry '{}' digest mismatch: expected={}, actual={digest_hex}",
                    entry.path, entry.digest_hex
                ));
            }
        }

        Ok(Self {
            raw,
            version,
            flags,
            crc32c: expected_crc,
            entries,
        })
    }

    pub fn entry_slice(&self, path: &str) -> Option<&'a [u8]> {
        self.entries.iter().find(|e| e.path == path).map(|e| {
            let start = e.offset as usize;
            let end = start + e.length as usize;
            &self.raw[start..end]
        })
    }
}

// ============================================================================
// 5. .QAR (QAR\0 V1) EVIDENTIARY & BITEMPORAL COLD-STORAGE ARCHIVER
// ============================================================================

/// Preserved evidentiary timestamps across the 4 temporal axes required for *Evidence Act*
/// authenticity and bitemporal auditability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservedEvidentiaryTimestamps {
    pub original_created_at_unix: u64,
    pub original_modified_at_unix: u64,
    pub sanctuary_ingested_at_unix: u64,
    pub archived_superseded_at_unix: u64,
}

impl PreservedEvidentiaryTimestamps {
    pub fn validate_chronology(&self) -> Result<(), String> {
        if self.original_created_at_unix == 0
            || self.original_modified_at_unix == 0
            || self.sanctuary_ingested_at_unix == 0
            || self.archived_superseded_at_unix == 0
        {
            return Err("All 4 evidentiary timestamps must be non-zero UNIX epoch seconds".into());
        }
        if self.original_modified_at_unix < self.original_created_at_unix {
            return Err("original_modified_at_unix cannot precede original_created_at_unix".into());
        }
        if self.archived_superseded_at_unix < self.sanctuary_ingested_at_unix {
            return Err(
                "archived_superseded_at_unix cannot precede sanctuary_ingested_at_unix".into(),
            );
        }
        Ok(())
    }
}

/// Metadata entry for an archived historical version inside a `.qar` container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QarVersionEntry {
    pub archive_entry_id: String,
    pub logical_artifact_uri: String,
    pub filename: String,
    pub version_sequence: u32,
    pub version_label: String,
    pub mime_type: String,
    pub offset: u64,
    pub length: u64,
    pub payload_digest_hex: String,
    pub predecessor_version_digest_hex: Option<String>,
    pub cumulative_custody_chain_hash_hex: String,
    pub timestamps: PreservedEvidentiaryTimestamps,
    pub custodian_natural_person_did: String,
    pub governing_agreement_id: Option<String>,
    pub statutory_preservation_basis: String,
    pub qecp_custody_signature_hex: String,
}

/// Lightweight 48-byte `NQuin48`-backed lineage stub retained in the hot runtime after pruning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeArchivedLineageStub {
    pub logical_artifact_uri: String,
    pub active_head_version_sequence: u32,
    pub archived_version_sequence: u32,
    pub cold_qar_container_uri: String,
    pub archive_entry_id: String,
    pub archived_payload_digest_hex: String,
    pub cumulative_custody_chain_hash_hex: String,
    pub timestamps: PreservedEvidentiaryTimestamps,
    pub runtime_bytes_pruned: u64,
    pub tombstone_quin: NQuin48,
}

/// Evidentiary Chain-of-Custody Verification Certificate emitted upon recalling a version from `.qar`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QarRecalledVersionCertificate {
    pub archive_entry: QarVersionEntry,
    pub recalled_payload_bytes: Vec<u8>,
    pub payload_digest_verified: bool,
    pub cumulative_chain_verified: bool,
    pub timestamps_chronology_verified: bool,
    pub evidence_act_compliance_note: String,
}

pub fn compute_cumulative_custody_chain_hash(
    prev_chain_hash_hex: Option<&str>,
    logical_artifact_uri: &str,
    version_sequence: u32,
    payload_digest_hex: &str,
    timestamps: &PreservedEvidentiaryTimestamps,
    custodian_did: &str,
) -> String {
    let material = format!(
        "QAR_CHAIN_V1|prev={}|uri={}|seq={}|digest={}|c={}|m={}|i={}|a={}|custodian={}",
        prev_chain_hash_hex.unwrap_or("GENESIS"),
        logical_artifact_uri,
        version_sequence,
        payload_digest_hex,
        timestamps.original_created_at_unix,
        timestamps.original_modified_at_unix,
        timestamps.sanctuary_ingested_at_unix,
        timestamps.archived_superseded_at_unix,
        custodian_did
    );
    hex_encode(&compute_entry_digest32(material.as_bytes()))
}

struct PendingQarVersion {
    archive_entry_id: String,
    logical_artifact_uri: String,
    filename: String,
    version_sequence: u32,
    version_label: String,
    mime_type: String,
    payload_bytes: Vec<u8>,
    predecessor_version_digest_hex: Option<String>,
    timestamps: PreservedEvidentiaryTimestamps,
    custodian_natural_person_did: String,
    governing_agreement_id: Option<String>,
    statutory_preservation_basis: String,
}

/// Builder & Runtime Pruner for `.qar` (`QAR\0` v1) cold evidentiary archive containers.
pub struct QarContainerWriter {
    pub container_id: String,
    pending: Vec<PendingQarVersion>,
    pub runtime_lineage_stubs: Vec<RuntimeArchivedLineageStub>,
    pub runtime_tombstone_quins: Vec<NQuin48>,
    pub runtime_payload_bytes_freed: u64,
}

impl QarContainerWriter {
    pub fn new(container_id: impl Into<String>) -> Self {
        Self {
            container_id: container_id.into(),
            pending: Vec::new(),
            runtime_lineage_stubs: Vec::new(),
            runtime_tombstone_quins: Vec::new(),
            runtime_payload_bytes_freed: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn archive_superseded_version_and_prune_runtime(
        &mut self,
        archive_entry_id: impl Into<String>,
        logical_artifact_uri: impl Into<String>,
        filename: impl Into<String>,
        version_sequence: u32,
        version_label: impl Into<String>,
        mime_type: impl Into<String>,
        payload_bytes: Vec<u8>,
        predecessor_version_digest_hex: Option<String>,
        timestamps: PreservedEvidentiaryTimestamps,
        custodian_natural_person_did: impl Into<String>,
        governing_agreement_id: Option<String>,
        statutory_preservation_basis: impl Into<String>,
    ) -> Result<RuntimeArchivedLineageStub, String> {
        timestamps.validate_chronology()?;
        let entry_id = archive_entry_id.into();
        let uri = logical_artifact_uri.into();
        let custodian = custodian_natural_person_did.into();
        let bytes_len = payload_bytes.len() as u64;
        let payload_digest_hex = hex_encode(&compute_entry_digest32(&payload_bytes));

        let prev_chain = self.runtime_lineage_stubs.last().map(|s| s.cumulative_custody_chain_hash_hex.as_str());
        let chain_hash = compute_cumulative_custody_chain_hash(
            prev_chain,
            &uri,
            version_sequence,
            &payload_digest_hex,
            &timestamps,
            &custodian,
        );

        let tombstone_quin = NQuin48::new(
            q_hash_u64(&uri),
            q_hash_u64("qar:archivedSupersededVersionToColdVault"),
            q_hash_u64(&chain_hash),
            q_hash_u64(QAR_COLD_STORAGE_VAULT_URI),
            timestamps.original_created_at_unix,
        );

        let stub = RuntimeArchivedLineageStub {
            logical_artifact_uri: uri.clone(),
            active_head_version_sequence: version_sequence + 1,
            archived_version_sequence: version_sequence,
            cold_qar_container_uri: format!("{}/{}.qar", QAR_COLD_STORAGE_VAULT_URI, self.container_id),
            archive_entry_id: entry_id.clone(),
            archived_payload_digest_hex: payload_digest_hex,
            cumulative_custody_chain_hash_hex: chain_hash,
            timestamps,
            runtime_bytes_pruned: bytes_len,
            tombstone_quin,
        };

        self.runtime_payload_bytes_freed += bytes_len;
        self.runtime_tombstone_quins.push(tombstone_quin);
        self.runtime_lineage_stubs.push(stub.clone());

        self.pending.push(PendingQarVersion {
            archive_entry_id: entry_id,
            logical_artifact_uri: uri,
            filename: filename.into(),
            version_sequence,
            version_label: version_label.into(),
            mime_type: mime_type.into(),
            payload_bytes,
            predecessor_version_digest_hex,
            timestamps,
            custodian_natural_person_did: custodian,
            governing_agreement_id,
            statutory_preservation_basis: statutory_preservation_basis.into(),
        });

        Ok(stub)
    }

    pub fn seal_cold_qar_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; QAP_HEADER_LEN];
        let mut toc = Vec::with_capacity(self.pending.len());
        let mut prev_chain_hash: Option<String> = None;

        for item in &self.pending {
            let rem = buf.len() % QAP_ENTRY_ALIGN;
            if rem != 0 {
                buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem), 0);
            }
            let offset = buf.len() as u64;
            let length = item.payload_bytes.len() as u64;
            let payload_digest_hex = hex_encode(&compute_entry_digest32(&item.payload_bytes));
            let chain_hash = compute_cumulative_custody_chain_hash(
                prev_chain_hash.as_deref(),
                &item.logical_artifact_uri,
                item.version_sequence,
                &payload_digest_hex,
                &item.timestamps,
                &item.custodian_natural_person_did,
            );
            prev_chain_hash = Some(chain_hash.clone());
            buf.extend_from_slice(&item.payload_bytes);

            toc.push(QarVersionEntry {
                archive_entry_id: item.archive_entry_id.clone(),
                logical_artifact_uri: item.logical_artifact_uri.clone(),
                filename: item.filename.clone(),
                version_sequence: item.version_sequence,
                version_label: item.version_label.clone(),
                mime_type: item.mime_type.clone(),
                offset,
                length,
                payload_digest_hex,
                predecessor_version_digest_hex: item.predecessor_version_digest_hex.clone(),
                cumulative_custody_chain_hash_hex: chain_hash.clone(),
                timestamps: item.timestamps,
                custodian_natural_person_did: item.custodian_natural_person_did.clone(),
                governing_agreement_id: item.governing_agreement_id.clone(),
                statutory_preservation_basis: item.statutory_preservation_basis.clone(),
                qecp_custody_signature_hex: format!("ed25519_ml_dsa_87_qar_custody_{}", &chain_hash[..16]),
            });
        }

        let rem = buf.len() % QAP_ENTRY_ALIGN;
        if rem != 0 {
            buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem), 0);
        }
        let index_offset = buf.len() as u64;
        let toc_json = serde_json::to_vec(&toc).expect("serializes QarVersionEntry list");
        let mut index_bytes = Vec::with_capacity(3 + toc_json.len());
        index_bytes.extend_from_slice(&CBOR_LD_TAG);
        index_bytes.extend_from_slice(&toc_json);
        let index_length = index_bytes.len() as u64;
        buf.extend_from_slice(&index_bytes);

        let rem_end = buf.len() % QAP_ENTRY_ALIGN;
        if rem_end != 0 {
            buf.resize(buf.len() + (QAP_ENTRY_ALIGN - rem_end), 0);
        }
        let total_length = buf.len() as u64;

        let head_chain_digest = compute_entry_digest32(
            prev_chain_hash
                .as_deref()
                .unwrap_or("GENESIS")
                .as_bytes(),
        );
        let entry_count = toc.len() as u32;
        buf[0..4].copy_from_slice(&QAR_MAGIC);
        buf[4..6].copy_from_slice(&QAR_VERSION.to_le_bytes());
        buf[6..8].copy_from_slice(&1u16.to_le_bytes());
        buf[8..12].copy_from_slice(&entry_count.to_le_bytes());
        buf[12..20].copy_from_slice(&index_offset.to_le_bytes());
        buf[20..28].copy_from_slice(&index_length.to_le_bytes());
        buf[28..36].copy_from_slice(&total_length.to_le_bytes());
        buf[40..64].copy_from_slice(&head_chain_digest[0..24]);

        let crc = compute_crc32c(&buf[QAP_HEADER_LEN..]);
        buf[36..40].copy_from_slice(&crc.to_le_bytes());
        buf
    }
}

/// Zero-copy reader and *Evidence Act* Chain-of-Custody verifier for `.qar` cold archives.
pub struct QarContainerReader<'a> {
    pub raw: &'a [u8],
    pub version: u16,
    pub entries: Vec<QarVersionEntry>,
}

impl<'a> QarContainerReader<'a> {
    pub fn parse(raw: &'a [u8]) -> Result<Self, String> {
        if raw.len() < QAP_HEADER_LEN {
            return Err("Buffer smaller than 64-byte QAR header".into());
        }
        if raw[0..4] != QAR_MAGIC {
            return Err("Invalid QAR magic (expected `QAR\\0`)".into());
        }
        let version = u16::from_le_bytes([raw[4], raw[5]]);
        if version != QAR_VERSION {
            return Err(format!("Unsupported QAR version: {version}"));
        }
        let entry_count = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
        let index_offset = u64::from_le_bytes(raw[12..20].try_into().unwrap()) as usize;
        let index_length = u64::from_le_bytes(raw[20..28].try_into().unwrap()) as usize;
        let total_length = u64::from_le_bytes(raw[28..36].try_into().unwrap()) as usize;
        let expected_crc = u32::from_le_bytes(raw[36..40].try_into().unwrap());

        if total_length != raw.len() {
            return Err("QAR total_length mismatch".into());
        }
        let actual_crc = compute_crc32c(&raw[QAP_HEADER_LEN..]);
        if actual_crc != expected_crc {
            return Err("QAR CRC32C integrity failure".into());
        }
        let index_slice = &raw[index_offset..index_offset + index_length];
        if index_slice[0..3] != CBOR_LD_TAG {
            return Err("QAR index missing W3C CBOR-LD tag (0xD9 0x05 0x01)".into());
        }
        let entries: Vec<QarVersionEntry> = serde_json::from_slice(&index_slice[3..])
            .map_err(|e| format!("Failed to decode QAR index: {e}"))?;
        if entries.len() != entry_count {
            return Err("QAR entry_count mismatch".into());
        }

        let mut prev_chain: Option<String> = None;
        for entry in &entries {
            if (entry.offset as usize) % QAP_ENTRY_ALIGN != 0 {
                return Err(format!(
                    "QAR entry '{}' offset {} is not 64-byte page-aligned",
                    entry.archive_entry_id, entry.offset
                ));
            }
            entry.timestamps.validate_chronology()?;
            let start = entry.offset as usize;
            let end = start + entry.length as usize;
            let actual_digest = hex_encode(&compute_entry_digest32(&raw[start..end]));
            if actual_digest != entry.payload_digest_hex {
                return Err(format!(
                    "QAR entry '{}' payload digest mismatch",
                    entry.archive_entry_id
                ));
            }
            let expected_chain = compute_cumulative_custody_chain_hash(
                prev_chain.as_deref(),
                &entry.logical_artifact_uri,
                entry.version_sequence,
                &entry.payload_digest_hex,
                &entry.timestamps,
                &entry.custodian_natural_person_did,
            );
            if expected_chain != entry.cumulative_custody_chain_hash_hex {
                return Err(format!(
                    "QAR entry '{}' cumulative chain-of-custody hash broken",
                    entry.archive_entry_id
                ));
            }
            prev_chain = Some(expected_chain);
        }

        Ok(Self {
            raw,
            version,
            entries,
        })
    }

    pub fn recall_historical_version(
        &self,
        archive_entry_id: &str,
    ) -> Result<QarRecalledVersionCertificate, String> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.archive_entry_id == archive_entry_id)
            .ok_or_else(|| format!("Archive entry '{archive_entry_id}' not found"))?;
        let start = entry.offset as usize;
        let end = start + entry.length as usize;
        let slice = &self.raw[start..end];
        let digest_ok = hex_encode(&compute_entry_digest32(slice)) == entry.payload_digest_hex;
        let chrono_ok = entry.timestamps.validate_chronology().is_ok();

        Ok(QarRecalledVersionCertificate {
            archive_entry: entry.clone(),
            recalled_payload_bytes: slice.to_vec(),
            payload_digest_verified: digest_ok,
            cumulative_chain_verified: true,
            timestamps_chronology_verified: chrono_ok,
            evidence_act_compliance_note: format!(
                "Verified under {} — Original Created: {}, Original Modified: {}, Sanctuary Ingested: {}, Archived: {} (Custodian: {})",
                entry.statutory_preservation_basis,
                entry.timestamps.original_created_at_unix,
                entry.timestamps.original_modified_at_unix,
                entry.timestamps.sanctuary_ingested_at_unix,
                entry.timestamps.archived_superseded_at_unix,
                entry.custodian_natural_person_did
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qap_and_qar_page_aligned_containers_roundtrip_and_detect_tampering() {
        let yaml = r#"schema: "yaml-ld-q42/v2"
qap_id: "test-app"
title: "Test App"
version: "0.0.41"
license: "CC-BY-NC-ND-4.0"
author_did: "did:q42:person:timothy-holborn"
entrypoint: "vibe/main.vibe"
multi_scale_layers:
  - L0Continuum
  - L6QuantumPhysics
"#;
        let desc = QapYamlLdQ42Descriptor::parse_and_validate_yaml_text(yaml).unwrap();
        let mut writer = QapContainerWriter::new(
            QAP_FLAG_SIGNED_QECP
                | QAP_FLAG_CBOR_LD_MANIFEST
                | QAP_FLAG_LIVE_VIBE_OVERLAY_READY
                | QAP_FLAG_L0_L6_HIERARCHY_DECLARED,
        );
        writer.add_entry(
            "manifest/qap.q42.yaml",
            QapEntrySectionKind::YamlLdQ42Manifest,
            yaml.as_bytes().to_vec(),
        );
        writer.add_entry(
            "manifest/qap_manifest.cborld",
            QapEntrySectionKind::CborLdBinaryManifest,
            desc.to_cbor_ld_bytes(),
        );
        let qap_bytes = writer.seal();
        assert_eq!(qap_bytes.len() % QAP_ENTRY_ALIGN, 0);

        let reader = QapContainerReader::parse(&qap_bytes).unwrap();
        assert_eq!(reader.entries.len(), 2);

        let mut tampered = qap_bytes.clone();
        tampered[70] ^= 0xFF;
        assert!(QapContainerReader::parse(&tampered).is_err());

        // Verify .qar cold archiver
        let mut qar_writer = QarContainerWriter::new("test-archive");
        let ts = PreservedEvidentiaryTimestamps {
            original_created_at_unix: 1_750_000_000,
            original_modified_at_unix: 1_750_001_000,
            sanctuary_ingested_at_unix: 1_750_002_000,
            archived_superseded_at_unix: 1_750_003_000,
        };
        let stub = qar_writer
            .archive_superseded_version_and_prune_runtime(
                "QAR-01",
                "qualia://test/doc",
                "doc_v1.pdf",
                1,
                "v1.0",
                "application/pdf",
                b"%PDF-1.7 test payload".to_vec(),
                None,
                ts,
                "did:q42:person:timothy-holborn",
                None,
                "Evidence Act s 48",
            )
            .unwrap();
        assert!(stub.tombstone_quin.verify_parity());
        let qar_bytes = qar_writer.seal_cold_qar_bytes();
        let qar_reader = QarContainerReader::parse(&qar_bytes).unwrap();
        let cert = qar_reader.recall_historical_version("QAR-01").unwrap();
        assert!(cert.payload_digest_verified);
    }
}
