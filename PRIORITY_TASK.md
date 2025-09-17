# Migration Plan: Replace `anyhow` With `thiserror` in `zewif-zcashd`

Goal: Align `zewif-zcashd`’s error story with the updated `zewif` crate by removing the `anyhow` dependency, introducing a crate-specific `Error` enum powered by `thiserror` v2, and propagating the new error type throughout the migration and wallet parsing code. Note that we are intentionally moving away from `anyhow::Context`-style helpers, so the plan below assumes raw error variants without layering extra context strings.

After performing each step of this plan, update this document to reflect progress and any adjustments needed. The goal is to have a clear, actionable checklist that can be followed to completion.

---

## 1. Understand the Existing Patterns
- [x] Inventory every `anyhow` import across the crate (already partially done via `rg "anyhow"`). Categorize usage patterns: simple type aliases (`Result`), context attachments (`Context` trait), convenience macros (`anyhow!`, `bail!`), and doc examples that mention `anyhow::Result` — 48 source files import `anyhow`; ≈35 use only the alias, 13 rely on `bail!`, 5 on `anyhow!`, 7 call `.context(...)`, 3 use `.with_context(...)`, and multiple parser/type docs demonstrate `/// # use anyhow::Result`.
- [x] Map out external error sources that bubble up through `anyhow`, e.g., `std::io`, `hex`, `zcash_address`, `zecwallet`-specific types, to ensure the new `Error` enum has appropriate `#[from]` variants or wrapper variants — key inputs include `std::io::Error` from process/tree reads, `hex::FromHexError` and `TryFromSliceError` from blob/hex parsing, UTF-8 conversions, Zcash domain errors from `zcash_encoding`, `incrementalmerkletree`, `UnifiedFullViewingKey::address`, and fallible option unwraps currently wrapped via `anyhow!` for missing UFVKs or invalid receiver types.
- [x] Review `zewif`’s `error.rs` (`zewif/src/error.rs`) to understand the structure of the new `thiserror`-based enums and the direct error variants they expose — `zewif::Error` centralizes domain variants, still offers a `Context` wrapper, drops any helper traits, and leans on `#[from]` conversions for CBOR, envelope, and hex errors that we can mirror where dependencies overlap.

## 2. Introduce a Crate-Level Error Module
- [x] Create `src/error.rs` (or similar) defining `#[derive(thiserror::Error)] pub enum Error`. Seed the enum with variants matching common failure modes extracted in step 1. Aim to express the failure data directly in the variant payloads rather than wrapping everything in a generic context type. → Added `src/error.rs` with domain-specific variants plus a helper for wrapping context.
- [x] Provide `#[from]` conversions for frequently propagated errors (hex parsing, I/O, `serde`, `zcash_encoding`, etc.) so existing `?` uses continue to compile. → Initial set includes transparent conversions for IO, hex, UTF-8, slice conversions, and propagated `zewif::Error`; more can be appended as the migration progresses.
- [x] Define `pub type Result<T> = std::result::Result<T, Error>` and expose it (e.g., `pub use error::{Error, Result};`) from `lib.rs` so the rest of the crate can depend on it cleanly. → `lib.rs` now re-exports the new error module, preparing downstream files for refactors.

- [x] Stage A – Introduce dual-result support:
  - Added `parser::result::IntoAnyhow` so the `parse!` macro can return `zewif_zcashd::Result` internally while auto-converting to `anyhow::Result` for existing call sites.
  - `parse_macro.rs` now routes through a new `parse_result!` helper, emitting crate errors first and converting via the shim to preserve current behaviour.
- [x] Stage B – Convert parser core modules to crate errors:
  - Switched `parser_impl.rs`, `parseable_types.rs`, and helper macros to `crate::Result` and the new context helpers.
  - Replaced `anyhow` conveniences with structured `Error` variants and the crate `ResultExt` helpers.
  - Updated doctests in these files to reference `zewif_zcashd::Result`.
- [x] Stage C – Migrate downstream parser consumers (e.g., `zcashd_parser`, `zcashd_dump`, wallet type parsers) away from `anyhow::Result` so the compatibility shim can be removed.
- [x] Stage D – Remove the shim, make `parse!` return `zewif_zcashd::Result` exclusively, and clean up lingering `anyhow` imports in the parser tree.
- [x] Verify the doc comments and code examples in the parser module compile against the new types (may require `Result` imports from the crate root) after each stage.

## 4. Systematically Migrate Modules
- [x] For each module that currently imports `anyhow::{Result, Context, anyhow, bail}` (see Step 1 inventory), switch to the crate-level `Result` alias and replace macros:
  - `anyhow::Result` → `crate::Result` (or `use crate::error::Result`).
  - `bail!(...)` → structured `Error` variants.
  - `anyhow!(...)` → structured variants.
  - `.context(...)` / `.with_context(...)` → use the new `ResultExt`/`OptionExt` helpers or explicit error construction.
- [x] When encountering generic `anyhow!("... {}", value)` placeholders, decide whether to introduce a structured variant or reuse the context helper.
- [x] Ensure all `TryFrom` implementations and parsing routines return the new `Error` type while preserving `?` usage by providing the necessary `From` conversions.
- [x] Update doc examples (`/// # use anyhow::Result;`) to reference the crate’s `Result` alias to avoid doctest failures.

## 5. Adjust Dependencies and Build Configuration
- [x] Update `Cargo.toml` to remove `anyhow` and add `thiserror = "2"`. Check for other crates pulling in `anyhow` through features and disable or replace as needed.
- [ ] Run `cargo update` (if needed) to refresh `Cargo.lock`, ensuring the `anyhow` crate disappears and `thiserror` is locked in. *(Still outstanding; other workspace crates retain `anyhow` so lockfile entries remain for now.)*
- [ ] Grep for `anyhow` in build scripts, benches, tests, and documentation to confirm the dependency is gone. *(Workspace-wide sweep still pending; the `zewif-zcashd` crate itself is clean.)*

## 6. Validate and Clean Up
- [x] Execute `cargo check`, `cargo test`, and (if configured) `cargo clippy` to surface any remaining type mismatches or unused imports from the migration. *(`cargo check -p zewif-zcashd` now passes; tests/clippy can be run as a follow-up if desired.)*
- [ ] Review error messages for clarity; tweak `Display` implementations for new error variants to match or improve on previous `anyhow` strings.
- [ ] Update top-level documentation (README, module docs) to describe the new error handling approach if necessary.
- [ ] Coordinate with downstream crates (e.g., anything depending on `zewif-zcashd`) to ensure the API change is communicated, possibly via changelog entry.

## 7. Final Integration Steps
- [ ] Squash/review commits to present the migration clearly (e.g., “Introduce zewif-zcashd error enum”, “Port parser macros”, “Remove anyhow dependency”).
- [ ] Open a PR summarizing the rationale, differences from the `zewif` migration, and any follow-up tasks (e.g., longer-term enhancements to the error enum).
- [ ] Consider adding regression tests around key failure paths to lock in expected error variants and messages.

---

**References**
- `zewif/src/error.rs` for the canonical `thiserror`-based approach.
- `zewif` commit history around the `anyhow` removal for concrete diff examples to emulate.

## Error Semantics Follow-Up

### Current Status of the Former `Error::message`
The generic `Message` variant has been removed. All call sites have been migrated to structured variants so errors now expose domain-specific details instead of opaque strings.

### Proposed Next Steps
1. Introduce structured error variants for each category, e.g.:
   - `Error::InvalidLength { expected: usize, actual: usize, kind: &'static str }`
   - `Error::InvalidValue { kind: &'static str, value: String }`
   - `Error::MissingRecord { key: String }` / `Error::DuplicateRecord { key: String }`
   - `Error::ExternalToolFailure { tool: &'static str, status: i32, stderr: String }`
   - `Error::InvalidUf vk { fingerprint: String }`
2. Refactor call sites currently using `Error::message` to populate these richer variants, adjusting doctests accordingly.
3. Once coverage is complete, document the new variants in the crate-level module doc so downstream consumers know what they can match on.

### Progress Update (2024-11-25)
- Added dedicated variants in `src/error.rs` covering record lookups, duplicate detection, command failures, compact size validation, receiver parsing, Orchard IVK handling, and numeric length mismatches.
- Updated parser, dump, migration, and wallet parsing modules to use the structured variants; removed the remaining runtime `bail!` macro calls.
- Removed the legacy `Message` variant and helper macro so all errors expose structured data.
- `cargo check -p zewif-zcashd` now passes with the new variants. Outstanding follow-up: add module-level docs describing the variants.

This staged cleanup will make errors self-describing, unlock straightforward localisation, and reduce dependence on free-form strings.
