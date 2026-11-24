# QAP Compiler (`qap-compiler` / `qapc`) — Agent Coordination & Licensing Guide

## Intellectual Property & Licensing Notice

QualiaDB, Webizen, `qap-compiler` (`qapc`), and all Qualia-specific architectures, integrations, modifications, and extensions are governed by:

**Copyright (c) 2026 Timothy Charles Holborn**  
**https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)  
Full terms: [`LICENSE`](LICENSE), [`LICENSE-QUALIA`](LICENSE-QUALIA), and [Creative Commons Legal Code](https://creativecommons.org/licenses/by-nc-nd/4.0/legalcode).

All participating AI agents and contributors must strictly honor this copyright, moral rights (UDHR Article 27(2) / ICESCR Article 15(1)(c)), and non-commercial attribution. Never replace root `LICENSE` or `Cargo.toml` license declarations with Apache-2.0 or MIT. Never use the term `"sovereign"` or `"self-sovereign"`; always use human-rights agreement vocabulary (**human rights**, **human dignity**, **human agency**, **natural person**, **moral and material interests of the author**, **sanctuary / selfhood**, and **Permissive Commons**).

---

## Architecture & Purpose

`C:\github\qap-compiler` is the standalone compiler, validator, signer, and live VibeScript packaging engine for:
1. **`.qap` (`QAP\0` v1) — Qualia Application Package Container**:
   - Transparent, 64-byte page-aligned (`QAP_ENTRY_ALIGN = 64`), zero-copy `mmap`-ready container for native desktop, mobile, and Webizen host applications (`poet.qap`, `wellfair.qap`, `cooperative-projects.qap`, etc.).
   - Embeds the human-readable `yaml-ld-q42` v2 descriptor (`manifest/qap.q42.yaml`), deterministic W3C CBOR-LD (`0xD9 0x05 0x01`) binary manifest (`manifest/qap_manifest.cborld`), live hot-reloadable `.vibe` scripts (`vibe/*.vibe`), `.q42` 48-byte `NQuin48` ontology vaults, `.10d` Holographic 10D v2 (`HOLO` `0x0306`) assets, and QECP (`Ed25519 + ML-DSA-87`) hybrid post-quantum signatures.
   - Enforces the 7-Layer Multi-Scale Computational Hierarchy (`L0`–`L6`, where `L6 = L6QuantumPhysics`), the No-ASCII Policy (UTF-8/UTF-16 text + CBOR-LD binary sections), and the Sanctuary Staging Quarantine invariant (`collect != activate` in `sanctuary://qap/staging/collected`).
2. **`.qar` (`QAR\0` v1) — Fundamental Evidentiary & Bitemporal Cold-Storage Archive Container**:
   - Preserves historical file and package versions byte-for-byte on 64-byte page-aligned boundaries with 4 bitemporal/filesystem timestamps (`original_created_at_unix`, `original_modified_at_unix`, `sanctuary_ingested_at_unix`, `archived_superseded_at_unix`), *Evidence Act* (ss 48, 69, 146, 147, 161) digital evidence informatics, and an unbroken cumulative chain-of-custody hash link while pruning superseded payload bytes from the 42 MiB Prolog Sentinel hot runtime.
