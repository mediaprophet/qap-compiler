# `qap-compiler` (`qapc`) — Qualia Application Package (`.qap`) & Evidentiary Archive (`.qar`) Compiler

**Copyright (c) 2026 Timothy Charles Holborn**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (`CC BY-NC-ND 4.0`)

`qap-compiler` is the canonical standalone compiler, validator, QECP hybrid post-quantum signer, and live VibeScript packaging engine for Qualia Application Packages (`.qap` / `QAP\0` v1) and Fundamental Evidentiary & Bitemporal Cold-Storage Archives (`.qar` / `QAR\0` v1).

---

## Workspace Crates

| Crate | Role |
|---|---|
| [`crates/qap-format-core`](crates/qap-format-core/src/lib.rs) | Canonical `.qap` (`QAP\0` v1) and `.qar` (`QAR\0` v1) 64-byte page-aligned binary container writers, zero-copy readers, `yaml-ld-q42` v2 (`qap.q42.yaml`) validator (`L0`–`L6`, `L6QuantumPhysics`), W3C CBOR-LD (`0xD9 0x05 0x01`) encoder, and 48-byte `NQuin48` Sentinel receipt engine |
| [`crates/qap-compiler-engine`](crates/qap-compiler-engine/src/lib.rs) | Workspace scanner (`compile_workspace_to_qap`), VibeScript (`../qualiaDB/crates/vibe`) grammar validator & zero-compile live runtime evaluator, Sanctuary staging quarantine gate (`collect != activate` in `sanctuary://qap/staging/collected`), and QECP (`Ed25519 + ML-DSA-87`) signer |
| [`crates/qapc-cli`](crates/qapc-cli/src/main.rs) | `qapc` CLI (`build`, `inspect`, `activate-eval`, `build-all`) |

---

## CLI Usage

```powershell
# Compile all 3 flagship native repositories (poet.qap, wellfair.qap, cooperative-projects.qap)
# and seal an Evidence Act cold archive (dist/ecosystem-qap-lineage.qar)
cargo run -p qapc-cli -- build-all -d dist

# Compile a single workspace into a page-aligned .qap package
cargo run -p qapc-cli -- build C:\github\wellfair -o dist\wellfair.qap

# Zero-copy inspect a .qap container header, 64-byte page alignment, and TOC entries
cargo run -p qapc-cli -- inspect dist\wellfair.qap

# Activate a staged .qap by a natural-person principal and evaluate its live .vibe entrypoint
cargo run -p qapc-cli -- activate-eval C:\github\cooperative-projects did:q42:person:timothy-holborn

# Check version pins, minor-update tolerance, and capability API surface contracts
cargo run -p qapc-cli -- check-env C:\github\wellfair exact
cargo run -p qapc-cli -- check-env C:\github\wellfair minor-update
cargo run -p qapc-cli -- check-env C:\github\wellfair api-break
```
