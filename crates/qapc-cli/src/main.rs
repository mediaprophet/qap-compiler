//! `qapc`: Canonical Qualia Application Package (`.qap` / `QAP\0` v1) & Evidentiary Cold-Storage
//! Archive (`.qar` / `QAR\0` v1) Compiler CLI.
//!
//! Copyright (c) 2026 Timothy Charles Holborn. All rights reserved.
//! License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0).

use qap_compiler_engine::compile_workspace_to_qap;
use qap_format_core::{
    HostEnvironmentProfile, PreservedEvidentiaryTimestamps, QapContainerReader,
    QarContainerReader, QarContainerWriter, WebizenNativeProfileKind, QAP_ENTRY_ALIGN,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn print_usage() {
    println!("qapc — Qualia Application Package (.qap) & Evidentiary Archive (.qar) Compiler (v0.0.41)");
    println!("Copyright (c) 2026 Timothy Charles Holborn | License: CC BY-NC-ND 4.0");
    println!();
    println!("Usage:");
    println!("  qapc build <workspace-dir> [-o <out.qap>]       Compile workspace (qap.q42.yaml + .vibe + .q42) to .qap");
    println!("  qapc inspect <file.qap>                         Zero-copy inspect .qap header, page alignment & TOC entries");
    println!("  qapc activate-eval <workspace-dir> [did]        Activate staged .qap (collect != activate) & evaluate live .vibe");
    println!("  qapc check-env <workspace-dir> [mode] [--profile <host|desktop|subsystem|mobile|iot-headless|iot-display>]");
    println!("                                                 Check version pins, minor-update tolerance & API surface contracts (mode: exact | minor-update | api-break)");
    println!("  qapc build-all [-d <dist-dir>]                  Compile poet.qap, wellfair.qap & cooperative-projects.qap + .qar archive");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "build" => {
            let ws_arg = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let ws_path = PathBuf::from(ws_arg);
            let pkg = compile_workspace_to_qap(&ws_path).unwrap_or_else(|e| {
                eprintln!("qapc build error: {e}");
                process::exit(1);
            });
            let out_path = if let Some(pos) = args.iter().position(|a| a == "-o") {
                PathBuf::from(args.get(pos + 1).expect("expected path after -o"))
            } else {
                PathBuf::from(format!("{}.qap", pkg.descriptor.qap_id))
            };
            if let Some(parent) = out_path.parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            fs::write(&out_path, &pkg.qap_bytes).expect("writes .qap container");
            let reader = QapContainerReader::parse(&pkg.qap_bytes).expect("verifies .qap");
            println!(
                "Sealed '{}' -> {} ({} bytes, {} 64B-aligned entries, magic QAP\\0 v{}, CRC32C {:#010x}, digest {})",
                pkg.descriptor.qap_id,
                out_path.display(),
                pkg.qap_bytes.len(),
                reader.entries.len(),
                reader.version,
                reader.crc32c,
                &pkg.package_digest_hex[..24]
            );
        }
        "inspect" => {
            let file_arg = args.get(2).unwrap_or_else(|| {
                eprintln!("Usage: qapc inspect <file.qap>");
                process::exit(1);
            });
            let raw = fs::read(file_arg).unwrap_or_else(|e| {
                eprintln!("Failed to read '{file_arg}': {e}");
                process::exit(1);
            });
            let reader = QapContainerReader::parse(&raw).unwrap_or_else(|e| {
                eprintln!("Invalid .qap container '{file_arg}': {e}");
                process::exit(1);
            });
            println!(
                "QAP Container: {} | Magic: QAP\\0 v{} | Flags: {:#06x} | Total Size: {} B (64B-Aligned: {}) | CRC32C: {:#010x}",
                file_arg,
                reader.version,
                reader.flags,
                raw.len(),
                raw.len() % QAP_ENTRY_ALIGN == 0,
                reader.crc32c
            );
            for (idx, entry) in reader.entries.iter().enumerate() {
                println!(
                    "  [{idx:02}] {:<36} | {:<24?} | offset={:<6} len={:<6} | sha256={}…",
                    entry.path,
                    entry.section_kind,
                    entry.offset,
                    entry.length,
                    &entry.digest_hex[..16]
                );
            }
        }
        "activate-eval" => {
            let ws_arg = args.get(2).map(|s| s.as_str()).unwrap_or(".");
            let principal_did = args
                .get(3)
                .map(|s| s.as_str())
                .unwrap_or("did:q42:person:timothy-holborn");
            let mut pkg = compile_workspace_to_qap(Path::new(ws_arg)).unwrap_or_else(|e| {
                eprintln!("qapc activate-eval compile error: {e}");
                process::exit(1);
            });
            let quin = pkg
                .activate_by_natural_person(principal_did)
                .unwrap_or_else(|e| {
                    eprintln!("qapc activation failed: {e}");
                    process::exit(1);
                });
            println!(
                "Activated Qap '{}' by '{}' -> State: {:?} | 48-Byte NQuin48 Parity: {} | Entrypoint '{}' Eval: {}",
                pkg.descriptor.qap_id,
                principal_did,
                pkg.state,
                quin.verify_parity(),
                pkg.descriptor.entrypoint_vibe,
                pkg.last_vibe_eval_summary
                    .as_deref()
                    .unwrap_or("(declarative scene/workbench manifest verified)")
            );
        }
        "check-env" => {
            let mut ws_arg = ".".to_string();
            let mut mode = "minor-update".to_string();
            let mut profile_kind: Option<WebizenNativeProfileKind> = None;
            let mut i = 2usize;
            while i < args.len() {
                let a = args[i].as_str();
                if a == "--profile" {
                    let raw = args.get(i + 1).map(|s| s.as_str()).unwrap_or("");
                    profile_kind = WebizenNativeProfileKind::parse_flag(raw);
                    if profile_kind.is_none() {
                        eprintln!(
                            "qapc check-env: unknown --profile '{raw}' (expected host|desktop|subsystem|mobile|iot-headless|iot-display)"
                        );
                        process::exit(1);
                    }
                    i += 2;
                    continue;
                }
                if let Some(raw) = a.strip_prefix("--profile=") {
                    profile_kind = WebizenNativeProfileKind::parse_flag(raw);
                    if profile_kind.is_none() {
                        eprintln!(
                            "qapc check-env: unknown --profile '{raw}' (expected host|desktop|subsystem|mobile|iot-headless|iot-display)"
                        );
                        process::exit(1);
                    }
                    i += 1;
                    continue;
                }
                if matches!(a, "exact" | "minor-update" | "api-break") {
                    mode = a.to_string();
                    i += 1;
                    continue;
                }
                if !a.starts_with('-') && ws_arg == "." {
                    ws_arg = a.to_string();
                    i += 1;
                    continue;
                }
                eprintln!("qapc check-env: unexpected argument '{a}'");
                process::exit(1);
            }

            let pkg = compile_workspace_to_qap(Path::new(&ws_arg)).unwrap_or_else(|e| {
                eprintln!("qapc check-env compile error: {e}");
                process::exit(1);
            });

            let apply_api_break = |env: &mut HostEnvironmentProfile| {
                env.environment_label =
                    "Simulated Host with Breaking API Surface Change".to_string();
                if let Some(first_contract) = pkg
                    .descriptor
                    .compatibility_matrix
                    .capability_surface_contracts
                    .first()
                {
                    if let Some(host_cap) = env
                        .available_capabilities
                        .iter_mut()
                        .find(|c| c.capability_id == first_contract.capability_id)
                    {
                        host_cap.provided_output_fields.clear();
                    }
                }
            };

            let apply_minor = |env: &mut HostEnvironmentProfile| {
                for comp in &mut env.installed_components {
                    match comp.component_id.as_str() {
                        "qualiadb-core" | "aether-ui" => {
                            comp.installed_version = "0.0.43.5".to_string()
                        }
                        "vibescript-engine" => comp.installed_version = "0.1.2".to_string(),
                        "etalon-gpu" => comp.installed_version = "0.0.4".to_string(),
                        "webizen-synth-morphisms" => {
                            comp.installed_version = "0.0.42.10".to_string()
                        }
                        _ => {}
                    }
                }
            };

            let host_env = match (profile_kind, mode.as_str()) {
                (Some(kind), "exact") => HostEnvironmentProfile::for_debian_profile(kind),
                (Some(kind), "api-break") => {
                    let mut env = HostEnvironmentProfile::for_debian_profile(kind);
                    apply_api_break(&mut env);
                    env
                }
                (Some(kind), _) => {
                    let mut env = HostEnvironmentProfile::for_debian_profile(kind);
                    env.environment_label = format!(
                        "User Host Environment with Minor/Patch Updates on {}",
                        kind.slug()
                    );
                    apply_minor(&mut env);
                    env
                }
                (None, "exact") => HostEnvironmentProfile::baseline_pinned_2026_11(),
                (None, "api-break") => {
                    let mut env = HostEnvironmentProfile::minor_updated_host_environment();
                    apply_api_break(&mut env);
                    env
                }
                (None, _) => HostEnvironmentProfile::minor_updated_host_environment(),
            };

            let report = pkg.check_host_environment_compatibility(&host_env);
            println!(
                "QAP Compatibility Report for '{}' (v{}) against '{}':",
                report.qap_id, pkg.descriptor.version, report.host_environment_label
            );
            println!(
                "  Overall Status: {:?} | Can Compile & Activate: {} | NQuin48 Receipt Parity: {}",
                report.overall_status,
                report.can_compile_and_activate,
                report.compatibility_receipt_quin.verify_parity()
            );
            for f in &report.findings {
                println!(
                    "  - [{:<30}] {:<36} | Designed: {:<32} | Host: {:<22} | {}",
                    format!("{:?}", f.status),
                    f.subject_id,
                    f.designed_for,
                    f.host_actual,
                    f.detail_and_remedy
                );
            }
        }
        "build-all" => {
            let dist_dir = if let Some(pos) = args.iter().position(|a| a == "-d") {
                PathBuf::from(args.get(pos + 1).expect("expected path after -d"))
            } else {
                PathBuf::from("dist")
            };
            fs::create_dir_all(&dist_dir).expect("creates dist directory");

            let candidate_targets = [
                "examples/reference-qap",
                "examples/iot-robotics-headless-qap",
                "examples/iot-shop-signage-display-qap",
                "C:/github/poet",
                "C:/github/wellfair",
                "C:/github/cooperative-projects",
            ];
            let targets: Vec<&str> = candidate_targets
                .into_iter()
                .filter(|p| Path::new(p).join("qap.q42.yaml").exists())
                .collect();

            let mut qar_writer = QarContainerWriter::new("ecosystem-qap-release-lineage-20261124");
            let mut prev_digest: Option<String> = None;

            for (seq, ws_str) in targets.iter().enumerate() {
                let ws_path = PathBuf::from(ws_str);
                let mut pkg = compile_workspace_to_qap(&ws_path).unwrap_or_else(|e| {
                    eprintln!("Failed to compile '{ws_str}': {e}");
                    process::exit(1);
                });
                let out_qap = dist_dir.join(format!("{}.qap", pkg.descriptor.qap_id));
                fs::write(&out_qap, &pkg.qap_bytes).expect("writes .qap");

                let entry_count = {
                    let reader = QapContainerReader::parse(&pkg.qap_bytes).expect("valid .qap");
                    reader.entries.len()
                };
                let quin = pkg
                    .activate_by_natural_person("did:q42:person:timothy-holborn")
                    .expect("activates .qap");

                println!(
                    "[{}] Compiled & Activated '{}' -> {} ({} B, {} entries, NQuin48 parity={}, Vibe eval={})",
                    seq + 1,
                    pkg.descriptor.qap_id,
                    out_qap.display(),
                    pkg.qap_bytes.len(),
                    entry_count,
                    quin.verify_parity(),
                    pkg.last_vibe_eval_summary.is_some()
                );

                let ts = PreservedEvidentiaryTimestamps {
                    original_created_at_unix: 1_763_900_000 + (seq as u64 * 100),
                    original_modified_at_unix: 1_763_950_000 + (seq as u64 * 100),
                    sanctuary_ingested_at_unix: 1_763_980_000 + (seq as u64 * 100),
                    archived_superseded_at_unix: 1_763_999_000 + (seq as u64 * 100),
                };
                let stub = qar_writer
                    .archive_superseded_version_and_prune_runtime(
                        format!("QAR-PKG-{:02}", seq + 1),
                        format!("qualia://qap/releases/{}", pkg.descriptor.qap_id),
                        format!("{}.qap", pkg.descriptor.qap_id),
                        (seq + 1) as u32,
                        pkg.descriptor.version.clone(),
                        "application/vnd.qualia.qap",
                        pkg.qap_bytes.clone(),
                        prev_digest.clone(),
                        ts,
                        "did:q42:person:timothy-holborn",
                        Some("AGR-COOP-CHARTER-2026".to_string()),
                        "Evidence Act ss 48, 146 & 147 (Canonical QAP Compiler Release Lineage)",
                    )
                    .expect("archives .qap into .qar");
                prev_digest = Some(stub.archived_payload_digest_hex);
            }

            let qar_bytes = qar_writer.seal_cold_qar_bytes();
            let out_qar = dist_dir.join("ecosystem-qap-lineage.qar");
            fs::write(&out_qar, &qar_bytes).expect("writes .qar cold archive");
            let qar_reader = QarContainerReader::parse(&qar_bytes).expect("verifies .qar");
            let cert = qar_reader
                .recall_historical_version("QAR-PKG-01")
                .expect("recalls poet.qap from cold .qar");
            println!(
                "Sealed Ecosystem Cold Evidentiary Archive -> {} ({} B, {} archived .qap packages, Recalled '{}' Verified: {})",
                out_qar.display(),
                qar_bytes.len(),
                qar_reader.entries.len(),
                cert.archive_entry.filename,
                cert.payload_digest_verified && cert.cumulative_chain_verified && cert.timestamps_chronology_verified
            );
        }
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
