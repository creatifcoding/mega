# Deno Migration Plan

## Current State Analysis

### Dependencies Referenced (Old Version)
The code currently references:
- `deno` crate (custom fork at `emacs-ng/deno` branch)
- `deno_core` (commented out)
- `deno_runtime` (commented out)
- `rusty_v8` version 0.32

### Latest Versions Available
- `deno_core` 0.371.0
- `deno_runtime` 0.229.0
- `rusty_v8` 0.32.1

## Key API Changes Identified

### 1. Deno Crate Structure
**Old**: Single `deno` crate with everything
**New**: Split into:
- `deno_core`: Core V8 bindings and module system
- `deno_runtime`: Runtime features (permissions, ops, workers)

### 2. Worker Initialization
**Old**:
```rust
let program_state = ProgramState::build(flags).await?;
let worker = create_main_worker(&program_state, main_module, permissions);
```

**New** (estimated based on modern Deno):
```rust
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;

let options = WorkerOptions {
    // Configuration here
    ...
};
let worker = MainWorker::bootstrap_from_options(main_module, permissions, options);
```

### 3. Permissions System
**Old**: `Permissions::from_options(&flags.into())`
**New**: `PermissionsContainer` with different API

### 4. Module Loading
**Old**: `program_state.file_fetcher.insert_cached(file)`
**New**: Different module loading mechanism via `deno_core::ModuleLoader`

### 5. File Fetcher
**Old**: Direct access via `program_state.file_fetcher`
**New**: Likely needs custom `deno_core::ModuleLoader` implementation

## Implementation Strategy

### Phase 1: Update Dependencies
1. Update `crates/js/Cargo.toml`:
   - Add `deno_core = "0.371"`
   - Add `deno_runtime = "0.229"`  
   - Update `rusty_v8 = "0.32.1"`
   - Remove commented deno crate references

### Phase 2: Fix Core Integration
1. Update `javascript.rs`:
   - Replace `deno::program_state::ProgramState` usage
   - Update `deno_runtime::worker::MainWorker` initialization
   - Update permissions API usage
   - Fix module loading mechanism

2. Update `subcommands.rs`:
   - Rewrite all subcommand functions for new API
   - Update worker creation
   - Update module execution

### Phase 3: Fix Module Loading
1. Implement custom `ModuleLoader` if needed
2. Update file caching mechanism
3. Update module resolution

### Phase 4: Testing
1. Test basic JS evaluation
2. Test TypeScript compilation
3. Test async operations
4. Test lisp-JS interop
5. Test deno CLI commands

## Risks and Challenges

1. **API Incompatibilities**: Many Deno internal APIs have changed significantly
2. **File Caching**: The file caching mechanism may need complete rewrite
3. **TypeScript**: TypeScript compilation integration may have changed
4. **Ops System**: The ops system has evolved - need to verify our bindings still work
5. **V8 Version**: V8 version compatibility may require code changes

## Decision Points

1. **Use Deno CLI as library?**: Modern Deno is primarily a CLI tool. We may need to:
   - Fork Deno again to expose needed internals
   - OR reimplement functionality using deno_core/deno_runtime primitives
   - OR use Deno CLI as external dependency

2. **Maintain compatibility?**: Should we maintain backward compatibility or accept breaking changes?

## Next Steps

1. Set up test environment with latest Deno crates
2. Create minimal working example with new APIs
3. Port existing functionality incrementally
4. Update documentation

## References

- [deno_core documentation](https://docs.rs/deno_core/)
- [deno_runtime documentation](https://docs.rs/deno_runtime/)
- [Deno source code](https://github.com/denoland/deno)
