# Deno Reimplementation Guide

This document provides a step-by-step guide for reimplementing Deno support in emacs-ng with the latest Deno versions.

## Overview

The goal is to restore JavaScript/TypeScript evaluation in emacs-ng using modern Deno crates (`deno_core` and `deno_runtime`) while maintaining compatibility with the existing Lisp interface.

## Phase 1: Core Dependencies and Minimal Worker

### Step 1.1: Update Cargo.toml

Update `crates/js/Cargo.toml`:

```toml
[dependencies]
emacs-sys.path = "../emacs-sys"
lisp-macros.path = "../lisp-macros"
lisp-util.path = "../lisp-util"
lsp-json.path = "../lsp-json"
libc.workspace = true
futures = "0.3"
serde_json = { version = "1.0", features = ["preserve_order"] }
tokio = { workspace = true, features = ["full"] }

# Updated Deno dependencies
deno_core = "0.371"
deno_runtime = "0.229"
url = "2.5"
```

### Step 1.2: Simplified Worker Initialization

The old code used `ProgramState` and `create_main_worker`. The new approach:

```rust
use deno_runtime::deno_permissions::PermissionsContainer;
use deno_runtime::worker::{MainWorker, WorkerOptions};
use deno_core::{ModuleSpecifier, JsRuntime};
use std::rc::Rc;
use std::sync::Arc;

fn create_worker(
    main_module: ModuleSpecifier,
    permissions: PermissionsContainer,
) -> Result<MainWorker, AnyError> {
    let options = WorkerOptions {
        module_loader: Rc::new(deno_runtime::deno_fs::FsModuleLoader),
        permissions,
        ..Default::default()
    };
    
    MainWorker::bootstrap_from_options(
        main_module,
        permissions,
        options,
    )
}
```

### Step 1.3: Replace EmacsMainJsRuntime Structure

Old structure had:
- `tokio_runtime`
- `deno_worker`  
- `program_state`
- Various state management fields

New structure (simplified):

```rust
struct EmacsMainJsRuntime {
    tokio_runtime: Option<tokio::runtime::Runtime>,
    deno_worker: Option<MainWorker>,
    within_runtime: bool,
    module_counter: u64,
    stacked_v8_handle: Option<*mut v8::HandleScope<'static>>,
    options: EmacsJsOptions,
    proxy_template: Option<v8::Global<v8::ObjectTemplate>>,
    within_toplevel: bool,
    tick_scheduled: bool,
}
```

Key changes:
- Remove `program_state` field
- Keep core runtime management
- Permissions now part of worker, not separate state

### Step 1.4: Permissions System Update

Old:
```rust
let permissions = Permissions::from_options(&flags.into());
```

New:
```rust
use deno_runtime::deno_permissions::{
    Permissions, PermissionsOptions, PermissionsContainer
};

// For allow-all:
let permissions = PermissionsContainer::allow_all();

// For custom permissions:
let permissions_options = PermissionsOptions {
    allow_net: Some(vec![]),  // None = deny, Some(vec![]) = allow all
    allow_read: Some(vec![]),
    allow_write: Some(vec![]),
    allow_run: Some(vec![]),
    ..Default::default()
};
let permissions = PermissionsContainer::new(Permissions::from_options(&permissions_options)?);
```

## Phase 2: Module Loading and Execution

### Step 2.1: Module Loading Without ProgramState

Old code used `program_state.file_fetcher.insert_cached()` to inject modules.

New approach using custom module loader:

```rust
use deno_core::{ModuleLoader, ModuleSource, ModuleSourceCode, ModuleType};
use std::pin::Pin;
use futures::future::Future;

struct EmacsModuleLoader {
    cached_modules: Arc<Mutex<HashMap<ModuleSpecifier, String>>>,
}

impl ModuleLoader for EmacsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: deno_core::ResolutionKind,
    ) -> Result<ModuleSpecifier, AnyError> {
        deno_core::resolve_import(specifier, referrer)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
        _requested_module_type: deno_core::RequestedModuleType,
    ) -> Pin<Box<dyn Future<Output = Result<ModuleSource, AnyError>>>> {
        let specifier = module_specifier.clone();
        let cached = self.cached_modules.clone();
        
        async move {
            // Check cache first
            if let Some(source) = cached.lock().unwrap().get(&specifier) {
                return Ok(ModuleSource::new(
                    ModuleType::JavaScript,
                    ModuleSourceCode::String(source.clone().into()),
                    &specifier,
                    None,
                ));
            }
            
            // Load from file system
            let path = specifier.to_file_path()
                .map_err(|_| anyhow::anyhow!("Invalid file path"))?;
            let source = tokio::fs::read_to_string(&path).await?;
            
            Ok(ModuleSource::new(
                ModuleType::JavaScript,
                ModuleSourceCode::String(source.into()),
                &specifier,
                None,
            ))
        }
        .boxed_local()
    }
}
```

### Step 2.2: Module Execution

Old:
```rust
worker.execute_module(&main_module).await?;
worker.run_event_loop().await?;
```

New (similar but different context):
```rust
// Execute module
let mod_id = worker.preload_main_module(&main_module).await?;
worker.evaluate_module(mod_id).await?;

// Run event loop
worker.run_event_loop(false).await?;
```

### Step 2.3: Inline Script Execution

For `eval_js` functionality:

```rust
// Old approach used execute_module with cached fake file
// New approach: direct script execution

let result = worker.execute_script(
    &format!("./$anon${}.js", counter),
    source_code.into(),
)?;
```

## Phase 3: V8 Bindings and Lisp Integration

### Step 3.1: V8 Handle Scope Access

The existing code for getting v8 scopes remains similar, but accessing the JsRuntime is different:

Old:
```rust
let runtime = &mut worker.js_runtime;
let context = runtime.global_context();
let scope = &mut v8::HandleScope::with_context(runtime.v8_isolate(), context);
```

New:
```rust
let js_runtime = &mut worker.js_runtime;
let context = js_runtime.main_context();
let scope = &mut js_runtime.handle_scope();
```

### Step 3.2: Binding Lisp Functions

The `v8_bind_lisp_funcs` function needs minor updates:

```rust
pub(crate) fn v8_bind_lisp_funcs(
    worker: &mut MainWorker,
) -> Result<(), AnyError> {
    let js_runtime = &mut worker.js_runtime;
    let scope = &mut js_runtime.handle_scope();
    let context = scope.get_current_context();
    let global = context.global(scope);
    
    // Rest remains the same: bind functions to global object
    bind_global_fn!(scope, global, lisp_invoke);
    // ... other bindings
    
    // Execute preliminary JS
    js_runtime.execute_script("prelim.js", include_str!("prelim.js"))?;
    
    Ok(())
}
```

## Phase 4: Subcommands

### Step 4.1: Rewrite Subcommands

Each subcommand needs to be rewritten. Example for `eval_command`:

```rust
pub(crate) async fn eval_command(
    code: String,
    ext: String,
    print: bool,
) -> Result<(), AnyError> {
    let main_module = ModuleSpecifier::parse(&format!("file:///./$deno$eval.{}", ext))?;
    let permissions = PermissionsContainer::allow_all();
    
    let source_code = if print {
        format!("console.log({})", code)
    } else {
        code
    };
    
    // Create module loader with cached code
    let module_loader = Rc::new(EmacsModuleLoader::new());
    module_loader.insert_cached(main_module.clone(), source_code);
    
    let options = WorkerOptions {
        module_loader,
        permissions: permissions.clone(),
        ..Default::default()
    };
    
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        permissions,
        options,
    )?;
    
    // Bind lisp functions
    v8_bind_lisp_funcs(&mut worker)?;
    
    // Execute module
    let mod_id = worker.preload_main_module(&main_module).await?;
    worker.evaluate_module(mod_id).await?;
    worker.run_event_loop(false).await?;
    
    Ok(())
}
```

### Step 4.2: REPL Command

REPL is more complex and may require using Deno's REPL functionality:

```rust
pub(crate) async fn run_repl() -> Result<(), AnyError> {
    // This requires deeper integration with Deno's REPL
    // May need to use deno_cli crate or implement custom REPL
    // For now, this is a placeholder showing the structure
    
    let main_module = ModuleSpecifier::parse("file:///./$deno$repl.ts")?;
    let permissions = PermissionsContainer::allow_all();
    
    let options = WorkerOptions {
        // REPL-specific options
        ..Default::default()
    };
    
    let mut worker = MainWorker::bootstrap_from_options(
        main_module,
        permissions,
        options,
    )?;
    
    v8_bind_lisp_funcs(&mut worker)?;
    
    // REPL loop would go here
    // This is complex and may require deno_cli crate
    
    Ok(())
}
```

## Phase 5: TypeScript Support

TypeScript compilation in newer Deno versions is handled differently:

### Option 1: Use SWC (Modern Deno Approach)

Modern Deno uses SWC for TypeScript compilation:

```rust
use deno_ast::{parse_module, ParseParams, MediaType, EmitOptions};

fn transpile_typescript(source: &str) -> Result<String, AnyError> {
    let parsed = parse_module(ParseParams {
        specifier: "file:///module.ts".to_string(),
        text_info: deno_ast::SourceTextInfo::from_string(source.to_string()),
        media_type: MediaType::TypeScript,
        capture_tokens: false,
        scope_analysis: false,
        maybe_syntax: None,
    })?;
    
    let transpiled = parsed.transpile(&EmitOptions::default())?;
    Ok(transpiled.text)
}
```

### Option 2: Rust-based Transpilation

For simpler cases, use the built-in transpilation:

```rust
// Worker options with TypeScript support
let options = WorkerOptions {
    // Enable TypeScript transpilation
    // This is handled automatically by the module loader in most cases
    ..Default::default()
};
```

## Phase 6: Integration Points

### Step 6.1: Update EmacsJsOptions

```rust
#[derive(Clone)]
struct EmacsJsOptions {
    tick_rate: f64,
    permissions: PermissionsContainer,  // Changed from Option<Permissions>
    error_handler: LispObject,
    inspect: Option<String>,
    inspect_brk: Option<String>,
    use_color: bool,
    loops_per_tick: EmacsUint,
}
```

### Step 6.2: Update js_initialize

```rust
#[lisp_fn]
pub fn js_initialize(args: &[LispObject]) -> LispObject {
    let ops = permissions_from_args(args);
    EmacsMainJsRuntime::set_options(ops.clone());
    js_init_sys(&ops)
        .map(|_| emacs_sys::globals::Qt)
        .unwrap_or_else(|e| {
            error!("JS Failed to initialize with error: {}", e);
        })
}

fn js_init_sys(js_options: &EmacsJsOptions) -> Result<(), AnyError> {
    init_tokio()?;
    init_worker(js_options)?;
    Ok(())
}

fn init_worker(js_options: &EmacsJsOptions) -> Result<(), AnyError> {
    if EmacsMainJsRuntime::is_main_worker_active() {
        return Ok(());
    }
    
    let main_module = ModuleSpecifier::parse("file:///init.js")?;
    let module_loader = Rc::new(EmacsModuleLoader::new());
    
    let options = WorkerOptions {
        module_loader,
        permissions: js_options.permissions.clone(),
        // Add inspector support if configured
        ..Default::default()
    };
    
    let mut worker = MainWorker::bootstrap_from_options(
        main_module,
        js_options.permissions.clone(),
        options,
    )?;
    
    v8_bind_lisp_funcs(&mut worker)?;
    EmacsMainJsRuntime::set_deno_worker(worker);
    
    Ok(())
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_eval() {
        let code = "1 + 1".to_string();
        let result = eval_command(code, "js".to_string(), true).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_typescript_eval() {
        let code = "const x: number = 42; x".to_string();
        let result = eval_command(code, "ts".to_string(), false).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

1. Test basic JS evaluation from Lisp
2. Test TypeScript compilation
3. Test async/await
4. Test lisp function calls from JS
5. Test JS function calls from Lisp
6. Test error handling

## Migration Checklist

- [ ] Update Cargo.toml dependencies
- [ ] Rewrite EmacsMainJsRuntime structure
- [ ] Implement custom ModuleLoader
- [ ] Update worker initialization
- [ ] Update permissions system
- [ ] Rewrite eval_js functions
- [ ] Rewrite eval_js_file functions
- [ ] Update v8_bind_lisp_funcs
- [ ] Rewrite subcommands (eval, run, repl, test)
- [ ] Test basic JS evaluation
- [ ] Test TypeScript compilation
- [ ] Test async operations
- [ ] Test lisp-JS interop
- [ ] Update documentation
- [ ] Performance testing

## Known Challenges

1. **TypeScript Compilation**: May need `deno_ast` crate
2. **REPL**: Complex feature, may need `deno_cli` or custom impl
3. **File Watcher**: For `--watch` flag, needs different API
4. **Inspector/Debugger**: Inspector API may have changed
5. **Coverage**: Code coverage API likely changed
6. **Module Caching**: Need robust caching strategy
7. **Import Maps**: Support may require additional work

## Resources

- [deno_core API docs](https://docs.rs/deno_core/)
- [deno_runtime API docs](https://docs.rs/deno_runtime/)
- [Deno source code](https://github.com/denoland/deno)
- [Deno embedding examples](https://github.com/denoland/deno/tree/main/runtime/examples)
- [deno_ast for TypeScript](https://docs.rs/deno_ast/)
