# AI Development Guide

This document defines repository-specific instructions for AI coding agents working on Apache Teaclave TrustZone SDK. Follow it together with the root `AGENTS.md` and any instructions closer to the files being changed.

Repository configuration, implementation, and CI workflows are authoritative when they differ from this guide. Use this guide for actionable workflows and engineering constraints; consult those sources when details change.

## 1. Repository Layout and Toolchain

### 1.1 Repository Structure

Identify the relevant package, target, and workflow before making changes.

- `crates/`: Rust SDK crates.
- `tools/`: development tools, including `cargo-optee`.
- `examples/ca/`: Client Application examples.
- `examples/ta/`: Trusted Application examples.
- `projects/`: larger example projects, including `projects/web3/eth_wallet`.
- `scripts/` and `ci/`: development and CI scripts.
- Root-level `*-unknown-optee.json` files: target specifications for relevant builds.
- `tests/`: QEMU and related integration-test scripts.

Do not assume that every Rust directory belongs to the same Cargo workspace or follows the same build and test procedure. The `projects/web3/eth_wallet` project has a separate build step in the standard-library example CI path; it is not covered by `make std-examples` alone.

### 1.2 Rust Toolchain

The repository's `rust-toolchain.toml` is the source of truth. It currently specifies:

- `channel = "nightly-2026-08-05"`
- `profile = "minimal"`
- Components: `rust-src`, `rustfmt`, `clippy`
- Targets: `aarch64-unknown-linux-gnu`, `arm-unknown-linux-gnueabihf`

These values are a reference snapshot. Read the actual file before changing toolchain assumptions or diagnosing version-dependent behavior. Do not assume a globally installed toolchain matches CI.

### 1.3 License Headers

CI checks license headers using `apache/skywalking-eyes/header@v0.8.0`. The `.licenserc.yaml` file defines the corresponding configuration and exclusions.

When adding or moving files, inspect the actual configuration and CI workflow rather than duplicating exclusion rules here. Run the applicable license-header check when a change may affect its results.

## 2. General Development Principles

- **Inspect before changing.** Read the relevant implementation, callers, tests, manifests, and CI configuration.
- **Prefer the smallest complete change.** Avoid unrelated refactoring, dependencies, abstractions, and formatting changes.
- **Preserve contracts.** Consider public API behavior, resource ownership, error semantics, feature boundaries, and compatibility.
- **Verify the actual invariant.** Names, comments, and type shapes are not substitutes for checking API contracts and implementation.
- **Keep verification claims precise.** Report only checks actually run and distinguish their results from unverified behavior.

## 3. Dependencies, Features, and Build Configuration

Before changing dependencies, Cargo features, lockfiles, or build configuration:

1. Identify the affected package and workspace.
2. Inspect manifests, feature declarations, target-specific dependencies, lockfiles, and relevant CI commands.
3. Check behavior with the intended features enabled and disabled, where applicable.
4. Prefer existing repository patterns unless there is a concrete reason to introduce another mechanism.
5. Verify the affected configuration with relevant builds or tests.

Keep dependencies in the crate that owns their functionality. In particular, `*-sys` crates should remain mappings of the underlying OP-TEE C interfaces rather than becoming containers for unrelated high-level abstractions.

Do not add dependencies merely to work around a compilation error without understanding its cause. Disabling a Cargo feature does not necessarily remove every associated dependency; inspect optional and target-specific dependency declarations.

Update lockfiles only when required by the change or build process. Review the diff for unrelated package updates and explain significant dependency changes in the final report.

## 4. Unsafe Rust, Concurrency, and Resource Management

### 4.1 Unsafe Code

Before introducing or modifying `unsafe` code:

- Identify the invariant that makes the operation valid.
- Check the relevant Rust and FFI contracts.
- Verify pointer validity, alignment, initialization, aliasing, lifetimes, and thread-safety assumptions as applicable.
- Keep the unsafe region as small as practical.
- Add or update a `SAFETY` comment that explains why the specific operation is sound.

A safety comment must state a verifiable invariant, not merely restate the operation or assert that it is safe. Check whether the invariant holds across all relevant call paths.

Do not introduce manual `Send` or `Sync` implementations merely to silence a compiler error. Establish why the type does not satisfy the trait bounds and whether the proposed design genuinely meets the safety requirements.

### 4.2 Concurrency and Synchronization

Choose synchronization based on actual access paths and required invariants.

- Identify shared state, its possible accessors, and operations that require mutual exclusion.
- Determine whether ownership and existing type structure already provide the required guarantees.
- Add locks, atomics, or other synchronization only when they address a demonstrated requirement.
- Avoid redundant synchronization and unnecessary poison-handling logic.
- Verify synchronization claims against the API contract and implementation.

For example, `Mutex::get_mut` does not acquire a runtime lock or provide runtime mutual exclusion. It accesses the protected value through an exclusive mutable reference.

Do not infer that a type is thread-safe merely because it is wrapped in `Arc`, or that an operation is synchronized merely because a mutex exists somewhere in the type. Trace where the lock is actually acquired and held.

### 4.3 FFI Lifetimes and Resource Ownership

For every FFI handle or resource, establish:

- Which object owns it and releases it.
- How long the underlying context and resource remain valid.
- Which operations may run concurrently.
- What happens if initialization, invocation, or cleanup fails.
- Whether the handle can be used after close or destruction of its parent context.

Keep Rust ownership consistent with the underlying C API. Do not expose paths that allow a resource to outlive its owner or be used after close.

If an API requires operations under a particular context or synchronization guard, enforce that requirement at the operation boundary rather than relying on callers to uphold an unenforced invariant.

### 4.4 Error Handling

Follow the affected API's error model and preserve meaningful error information.

Do not suppress failures, convert them into success-shaped values, or add fallback behavior merely to make a test pass. Document and test necessary fallbacks. Avoid redundant error-handling layers, but do not discard errors callers need to determine whether an operation succeeded.

## 5. Security-Sensitive Changes

Read `SECURITY.md` and [`docs/security-model.md`](security-model.md) when relevant to a security-sensitive change.

Treat changes involving unsafe code, FFI, cryptography, authentication, authorization, key handling, memory ownership, or cross-boundary data as security-sensitive.

For such changes:

1. Identify the security property that must be preserved.
2. Trace relevant data flow, ownership, validation, and failure paths.
3. Inspect callers and related implementations, not only the changed function.
4. Consider invalid inputs, cleanup, concurrency, and configuration-dependent behavior.
5. Add or update tests for security-relevant invariants where feasible.

Distinguish confirmed defects from hypotheses. Support findings with concrete code paths, violated invariants, or reproducible behavior. State important review limitations, such as configurations not exercised or build scripts and generated code not inspected.

## 6. Tests and Verification Evidence

### 6.1 Status and Scope

Use these four statuses in reports:

- **Passed** — the command completed and the relevant check succeeded.
- **Failed** — the command ran and the check failed.
- **Not run** — the check was not attempted.
- **Blocked** — a required check could not be completed because of an external prerequisite or environment limitation.

Match evidence to the claim: a build is not runtime evidence, a host test does not establish TA runtime behavior, and one feature configuration does not establish correctness across all configurations. When blocked, state the missing prerequisite and what remains unverified.

### 6.2 Unit Test Guarantee Comments

**Every unit test must include a `Guarantee:` or `Guarantees:` comment** describing the behavior or invariant the test guarantees.

Follow the existing convention in the affected crate and ensure the comment matches the assertions the test actually makes. Update it when changing the test's scope; do not claim behavior the test does not verify.

## 7. Formatting and Linting

Use the repository's configured toolchain and workspace-aware commands.

### 7.1 Formatting

For the crate workspace:

```bash
cd crates
cargo fmt --all --check
```

For the tools workspace:

```bash
cd tools
cargo fmt --all --check
```

Do not substitute an independent `rustfmt --check` invocation for `cargo fmt --all --check`; workspace commands use the relevant Cargo configuration. If formatting fails, format with the workspace-aware command, inspect the diff, and rerun the check.

### 7.2 Clippy

For the crate workspace:

```bash
cd crates
cargo clippy \
  --workspace \
  --exclude secure_db \
  --exclude rustls_provider \
  --target aarch64-unknown-linux-gnu \
  -- -D warnings
```

For `cargo-optee`:

```bash
cd tools
cargo clippy -p cargo-optee -- -D warnings
```

Preserve the CI target and exclusions unless the task requires a different configuration. Report alternative checks separately rather than implying they are identical to CI.

## 8. Build and Test Workflows

### 8.1 SDK Crate Tests and Cross-Compilation

The standard setup path, run from the repository root, is:

```bash
./setup.sh
source "$HOME/.cargo/env"
./build_optee_libraries.sh "$HOME"
export OPTEE_DIR="$HOME"
source environment
```

Before running builds or tests, inspect the actual shell environment and required paths, particularly `TA_DEV_KIT_DIR`, `OPTEE_CLIENT_EXPORT`, and `OPTEE_DIR`. This setup sequence is a standard path, not the only valid shell configuration. The `environment` script supports explicit `TARGET_TA`, `TARGET_HOST`, and `OPTEE_DIR` inputs; respect custom configurations and verify that they target the intended build.

Run the package tests from `crates/`:

```bash
cargo test -p optee-utee --features no_panic_handler
cargo test -p optee-teec
cargo test -p optee-utee-build
cargo test -p optee-utee --features std
```

These commands exercise distinct packages or feature configurations. A pass in one does not establish that the others passed.

The CI cross-compilation checks include:

```bash
cargo build -p optee-utee --target aarch64-unknown-linux-gnu
cargo build -p optee-teec --target aarch64-unknown-linux-gnu
```

Run these from `crates/` in an environment with the required target and OP-TEE dependencies configured.

### 8.2 OP-TEE Client System Test

The `optee-teec-systest` job uses an ARM64 host environment. It is not a generic host-independent test command.

Configure the client library path as required by that environment:

```bash
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$OPTEE_DIR/optee_client/export_arm64/usr/lib"
cd crates
cargo run -p optee-teec-systest
```

The Trusted Application system-test package also has a build check:

```bash
cd crates
cargo build -p optee-utee-systest
```

A successful build is not evidence that a system test ran successfully. If running outside the intended ARM64 environment, report the limitation rather than treating an incompatible environment as a source-code failure.

### 8.3 Examples and QEMU

Run example targets from the repository root.

For no-std examples:

```bash
make no-std-examples -j2
```

For std examples, prepare the std dependencies and configure the environment **before sourcing `environment`**:

```bash
./setup_std_dependencies.sh
export STD=y
source environment
make std-examples -j2
```

`STD=y` affects how `environment` derives `TARGET_TA`. If `environment` was already sourced with the no-std configuration, source it again after setting `STD=y` so the target configuration is recalculated. Do not assume setting `STD=y` immediately before `make` retroactively changes variables already derived by the earlier `source`.

The `projects/web3/eth_wallet` project has a separate build step and is not included in `make std-examples` alone:

```bash
cd projects/web3/eth_wallet
make
```

#### QEMU runtime tests

A successful example build does not establish QEMU runtime success. The repository's runtime-test path uses scripts under `tests/`, including `tests/setup.sh`, `tests/optee-qemuv8.sh`, and applicable `tests/test_*.sh` scripts. The reusable workflow invokes the QEMU-based test scripts through `ci/ci.sh`.

Start by inspecting the scripts and their usage from the repository root:

```bash
sed -n '1,220p' ci/ci.sh
sed -n '1,220p' tests/setup.sh
sed -n '1,260p' tests/optee-qemuv8.sh
```

Then run the setup, QEMU launcher, and applicable test scripts using the invocation and prerequisites defined by those scripts. Do not guess environment-variable names or assume every `test_*.sh` script can run independently. If QEMU or another prerequisite is unavailable, report the runtime test as blocked rather than treating a successful build as a substitute.

The repository also has a `test-build-on-dev-docker` CI job. When reproducing that job, inspect its current steps in `ci.yml`, including configuration switching and the `$RUST_STD_DIR` setup. Do not assume that the reusable example workflow covers this separate environment.

### 8.4 Tools Tests

Run `cargo-optee` tests from `tools/`:

```bash
cd tools
cargo test -p cargo-optee
```

Use the appropriate workspace for tool-specific formatting, linting, and testing. Check package manifests and CI configuration when additional system dependencies or test prerequisites may be required.

## 9. Code Review and Collaboration

Before finalizing a change, review the complete diff, including untracked files. Confirm that:

- The implementation solves the requested problem without unrelated changes.
- Public interfaces, feature behavior, error semantics, and resource ownership remain coherent.
- Tests and their `Guarantee:` or `Guarantees:` comments match the intended behavior.
- Documentation, manifests, lockfiles, generated files, and configuration remain consistent.
- Related callers, implementations, tests, and files have not been missed.

For parallel work, establish file ownership and inspect the worktree before editing, overwriting, or reverting files. After integrating contributions, review the combined diff, resolve incompatible assumptions, and verify the integrated result. Do not discard another contributor's changes merely because they differ from an earlier plan.

## 10. Git and Delivery

Before declaring a change complete:

1. Inspect `git status` and the full diff.
2. Confirm that only intended files changed.
3. Check for accidental generated files, temporary artifacts, unrelated formatting, and unintended lockfile updates.
4. Run relevant checks against the final worktree state.
5. Report results and remaining limitations.

Do not use destructive Git commands or overwrite existing work without understanding their impact. During rebases and conflict resolution, inspect both sides and preserve the intended behavior.

Commit messages should describe the final change and follow repository conventions. Do not claim to have committed, pushed, or changed repository history unless that action was actually performed.

## 11. Final Report

Keep the final report concise. Include:

- **Changes:** what changed and why.
- **Verification:** commands actually run and their statuses.
- **Limitations:** checks not run or blocked.
- **Security scope:** important areas reviewed or left unverified for security-sensitive changes.

Distinguish build, unit-test, cross-compilation, system-test, and QEMU runtime results. Do not imply broader coverage than the evidence supports.
