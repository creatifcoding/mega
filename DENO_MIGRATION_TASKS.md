# Deno API Migration Tasks

## Overview
Migrate javascript.rs from old `deno` crate (custom fork) to modern `deno_core 0.371` + `deno_runtime 0.229`.

## Current Status
- ✅ Dependencies updated in Cargo.toml
- ✅ Build system resolved (globals.h generated)
- ✅ emacs-sys compiles
- ⏳ **NOW**: Migrate ~40 old Deno API calls in javascript.rs

## Specific Changes Required

### 1. Update Imports (Lines ~1-25)
**OLD**:
```rust
// No explicit deno imports - implicitly using custom fork
```

**NEW**:
```rust
use deno_core::{JsRuntime, RuntimeOptions, ModuleSpecifier};
use deno_runtime::{
    worker::{MainWorker, WorkerOptions},
    permissions::PermissionsContainer,
};
use deno_ast::MediaType;
```

### 2. Remove ProgramState (Line 103, 219-224)
**OLD**:
```rust
program_state: Option<Arc<deno::program_state::ProgramState>>,
```

**NEW**:
Remove this field entirely. Configuration now handled via WorkerOptions