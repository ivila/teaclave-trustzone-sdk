# AI Development Guide

This guide defines repository-specific rules for AI coding agents working on Apache Teaclave TrustZone SDK.

Read this document before making code changes. For security reviews or security-sensitive changes, also read [`SECURITY.md`](../SECURITY.md) and [`docs/security-model.md`](security-model.md).

## 1. Repository Facts

### 1.1 Workspace layout

The repository root is **not a Cargo workspace**. Do not run `cargo fmt --all`, `cargo test --workspace`, or similar workspace commands from the repository root.

The repository contains independent Cargo workspaces:

| Directory | Purpose |
| --- | --- |
| `crates/` | Rust SDK crates, including `optee-utee` and `optee-teec` |
| `tools/` | Rust development tools, including `cargo-optee` |
| `examples/ca/` | Client Application examples |
| `examples/ta/` | Trusted Application examples |

Always run Cargo commands from the appropriate workspace directory. Check the relevant CI workflow before choosing a command or claiming CI-equivalent validation.

### 1.2 Toolchain

The repository pins its Rust toolchain in [`rust-toolchain.toml`](../rust-toolchain.toml). Use the pinned toolchain and its configured components rather than an arbitrary local stable or nightly version.

When a toolchain-related failure occurs, inspect the pinned version, installed targets and components, and the exact CI environment before changing code or configuration.

### 1.3 Source file license headers

The CI license check uses `.licenserc.yaml`. Markdown files (`*.md`) are exempt, but newly added source and configuration files generally require the Apache license header.

Before adding or modifying files such as `.rs`, `.sh`, and `.yml`, inspect neighboring files and follow the repository's license-header convention. Do not weaken the license check to make a change pass.

## 2. Understand Before Changing

Before editing:

1. Identify the relevant crate, TA/CA component, build path, and public API.
2. Read the implementation and its callers, related tests, feature declarations, and applicable CI workflow.
3. Determine whether the code is built for `std`, `no_std`, host, ARM32, or AArch64.
4. Check whether the behavior depends on OP-TEE, GlobalPlatform APIs, native libraries, generated code, build scripts, or proc macros.
5. Identify the intended behavior and compatibility constraints.

Prefer the smallest complete change that fixes the actual problem. Do not introduce abstractions, dependencies, synchronization, or public API changes without a concrete need.

## 3. Rust and Repository Conventions

- Follow the patterns already used in the relevant module and neighboring crates.
- Preserve `std`/`no_std`, feature, and target boundaries.
- Check `Cargo.toml`, feature definitions, and dependency relationships before adding or changing dependencies.
- Do not assume host behavior proves behavior on a real TEE or target architecture.
- Preserve public API and observable behavior unless a breaking change is explicitly intended.
- Treat unknown enum values, error codes, FFI results, and unsupported configurations deliberately; do not silently discard information merely to simplify an implementation.

## 4. Unsafe Rust, Auto Traits, and FFI

Unsafe code requires a concrete safety argument. Review the invariant, not just whether the code compiles.

### 4.1 Before adding or changing unsafe code

Establish:

1. Which operation is unsafe and why.
2. Which invariants make the operation sound.
3. Who establishes and maintains those invariants.
4. Whether aliasing, lifetime, ownership, initialization, or pointer-validity requirements are satisfied.
5. Whether concurrent access is possible and how it is synchronized.
6. Whether cleanup and failure paths preserve the invariants.
7. Which tests or other evidence validate the change.

Write a `SAFETY` comment that explains the relevant invariant and why it holds. A comment that merely restates the operation is insufficient.

### 4.2 `Send`, `Sync`, and synchronization

- Prefer automatically derived `Send` and `Sync` implementations when they are sound.
- Do not add `unsafe impl Send` or `unsafe impl Sync` merely to satisfy a compiler error.
- Analyze `Send` and `Sync` separately. Being safe to move between threads does not imply being safe to share concurrently.
- Keep any required unsafe implementation in the smallest private wrapper that can enforce its invariant.
- Verify the native API's threading contract before claiming a handle or context is thread-safe.

A `Mutex` only synchronizes accesses that actually use the lock. `Mutex::get_mut(&mut self)` accesses the protected value through exclusive mutable access; it does not acquire the mutex or provide runtime mutual exclusion. Do not add redundant mutexes or poison handling without a demonstrated need. Conversely, do not remove synchronization without proving it is unnecessary.

### 4.3 FFI resources and lifetimes

For resources such as TEEC contexts, sessions, operation handles, and native pointers, trace:

- Resource ownership and the lifetime of dependencies.
- Initialization, partial failure, and cleanup behavior.
- Whether native code retains pointers after a call returns.
- Whether resources can be closed or dropped while still in use.
- Whether multiple handles can access the same underlying native object concurrently.
- Whether error codes and cleanup failures are handled consistently.

Use RAII or another explicit ownership strategy where appropriate. An abstraction is useful only if its implementation actually enforces the claimed invariant.

Do not infer the thread-safety contract of OP-TEE or a native API solely from Rust types. Confirm it from the relevant API contract and implementation.

## 5. Security Model and Security Reviews

Read [`SECURITY.md`](../SECURITY.md) and [`security-model.md`](security-model.md) before security-sensitive work.

Apply the repository's TrustZone trust boundaries:

- Treat the Normal World and its inputs as untrusted where the security model requires it.
- Do not rely on Client Application validation as the security boundary for a Trusted Application.
- Validate inputs at the boundary where the security invariant must be enforced.
- Consider shared memory mutation and TOCTOU risks when data can change between validation and use.
- Do not report the mere presence of `unsafe`, raw pointers, or FFI as a vulnerability without demonstrating a violated invariant or reachable security impact.
- Consider relevant TA dependencies, build scripts, generated code, and cross-boundary data flows when defining review scope.

For each security finding, explain:

1. The affected component and trust boundary.
2. The attacker capability and relevant input or call path.
3. The violated invariant and realistic impact.
4. Why existing checks or API contracts do not prevent the issue.
5. A minimal remediation and a test that exercises the relevant behavior.

Clearly distinguish confirmed vulnerabilities from assumptions, hardening suggestions, and questions requiring further investigation. State unreviewed scope and environmental limitations. Follow the project's security-reporting policy.

## 6. Tests and Evidence

### 6.1 Test the claimed behavior

Add or update tests for the behavior changed, including relevant failure paths, edge cases, and resource lifecycles.

For FFI and resource-management changes, consider initialization failures, repeated operations, multiple resources, cleanup, and invalid inputs where applicable.

A test comment such as `Guarantee:` must describe only what the assertions actually prove. For example, a compile-time trait check does not prove runtime thread safety, and a host mock test does not prove the behavior of the native TEE implementation.

### 6.2 Report validation honestly

Separate checks into:

- **Passed**: executed successfully.
- **Failed**: executed and failed, with the relevant error.
- **Not run**: not executed.
- **Blocked**: could not be executed because of a missing tool, environment, target, or external dependency.

Do not claim that all cases, architectures, feature combinations, or security paths are covered unless the evidence supports that claim.

## 7. Formatting and Import Ordering

Use the workspace-aware commands from the relevant workspace directory. The repository's CI runs formatting checks separately for `crates/` and `tools/`.

For the SDK crates:

```bash
cd crates
cargo fmt --all --check
```

For development tools:

```bash
cd tools
cargo fmt --all --check
```

To apply formatting, run `cargo fmt --all` in the corresponding workspace, then inspect the diff.

Do not use standalone `rustfmt --check` as a substitute for the repository's Cargo formatting check. Do not guess import ordering or manually edit formatting diffs without rerunning the appropriate formatter.

If local formatting differs from CI, compare the exact command, the pinned Rust toolchain, the workspace being checked, and the relevant configuration. Do not assume a `rustfmt.toml` or `style_edition` configuration exists; first verify that the repository or invocation actually defines one.

## 8. Repository Validation Commands

Choose checks based on the changed component. Do not run every command blindly, and do not describe a partial check as equivalent to the full CI pipeline.

### 8.1 SDK crates

The following commands are based on `.github/workflows/reuse_test.yml`:

```bash
cd crates

cargo test -p optee-utee --features no_panic_handler
cargo test -p optee-teec
cargo test -p optee-utee-build
cargo test -p optee-utee --features std

cargo build -p optee-utee --target aarch64-unknown-linux-gnu
cargo build -p optee-teec --target aarch64-unknown-linux-gnu

cargo clippy --workspace \
  --exclude secure_db \
  --exclude rustls_provider \
  --target aarch64-unknown-linux-gnu -- -D warnings

cargo fmt --all --check
```

These commands may require the repository's configured environment, native dependencies, or installed target toolchains. Check the workflow and setup instructions before running them.

### 8.2 Development tools

The CI checks under `tools/` include:

```bash
cd tools

cargo test -p cargo-optee
cargo clippy -p cargo-optee -- -D warnings
cargo fmt --all --check
```

### 8.3 TA/CA examples and OP-TEE integration

Example builds and runtime tests depend on the configured OP-TEE environment and target architecture.

- Inspect `.github/workflows/ci.yml` and the relevant reusable workflow.
- The CI example matrix covers ARM32 and AArch64, with both `std` and `no_std` TA configurations.
- The workflow uses `make` for `no_std` examples and `make std-examples` for `std` examples in its configured emulator environment.
- `cargo optee` is part of the repository's development workflow; inspect the tool documentation and environment setup before choosing its arguments.
- `ci/ci.sh` runs integration tests under `tests/`. It uses paths relative to the `ci/` directory, so invoke it from that directory after setting up the required environment.

Do not claim QEMU, emulator, native OP-TEE, ARM32, or AArch64 validation unless the corresponding check was actually performed.

### 8.4 General diff checks

From the repository root, inspect the complete change:

```bash
git diff --check
git status --short
git diff
```

For new, untracked files, explicitly inspect their contents as well; `git diff` alone does not show them.

## 9. CI and Failure Diagnosis

When a check fails:

1. Capture the exact command and the first relevant error.
2. Reproduce the failure in the same workspace and, where practical, with the pinned toolchain.
3. Determine whether the cause is code, configuration, environment, toolchain, or test assumptions.
4. Make the smallest change that addresses the demonstrated cause.
5. Rerun the failed check and the relevant regression tests.

Do not disable lint rules, weaken assertions, remove test coverage, or change CI configuration merely to make a failure disappear. Change those constraints only when there is a justified, explicit reason.

Do not attribute a CI failure to a toolchain or environment mismatch without evidence.

## 10. Refactoring and Compatibility

Before refactoring, identify the public and internal contracts that must remain true.

Review changes to:

- Public types, enum variants, method signatures, and error behavior.
- `Send`/`Sync` behavior and resource lifetimes.
- Feature flags, dependency selection, and target support.
- FFI calls, initialization, and cleanup ordering.
- Generated code, build scripts, and examples that consume the API.

Keep unrelated cleanup separate from correctness fixes when practical. Do not introduce a new abstraction solely to make a diff look more uniform.

## 11. Parallel Work and Shared Changes

When using multiple agents or concurrent work:

- Give each agent a clear scope and expected output.
- Inspect repository status before editing.
- Do not overwrite another agent's changes or assume the worktree is isolated.
- Review the integrated diff for conflicting assumptions, duplicated logic, and missing tests.
- Run the relevant checks after integration.

Treat agent-generated findings and patches as hypotheses to verify, not as proof of correctness.

## 12. Git and Completion Report

Before finishing:

1. Inspect the complete diff, including untracked files.
2. Check for accidental changes, unrelated files, debug output, and generated artifacts.
3. Confirm relevant tests and checks have been run, or clearly state why they were not.
4. Summarize the behavior changed, the reasoning behind it, and any remaining risks.
5. Report commands and results accurately.

Do not stage, commit, rewrite history, or discard existing user changes unless explicitly requested.

Apache project contributions do not require a `Signed-off-by` trailer. Such trailers are optional and do not replace any applicable Apache Contributor License Agreement requirements.

## 13. Completion Criteria

A change is ready to report as complete when:

- The implementation addresses the demonstrated problem.
- Relevant invariants, API behavior, and security boundaries have been reviewed.
- Tests cover the changed behavior and important failure paths.
- Formatting and applicable lint checks pass, or limitations are stated.
- The final diff contains no unexplained or unrelated changes.
- The completion report distinguishes verified results from assumptions and unexecuted checks.
