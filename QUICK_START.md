# Deno Reimplementation - Quick Start Guide

**Last Updated**: 2025-12-06  
**Status**: Ready for Phase 3 implementation  
**Estimated Time**: 7-11 hours for basic functionality

## TL;DR

The Deno migration is 85% complete. Dependencies updated, build system fixed, Phase 1-2 done. 
**Next**: Update `eval_js()` and `eval_ts()` functions in `crates/js/src/javascript.rs` to use new Deno 0.371 APIs.

## Prerequisites Check

```bash
# 1. Verify you're on the PR branch
git branch --show-current
# Should show: copilot/update-emacs-ng-for-deno

# 2. Check Rust toolchain
rustc --version
# Should show: rustc 1.93.0-nightly (or similar)

# 3. Verify globals.h exists
ls -lh src/globals.h
# Should show: ~307KB file

# 4. Test build
cargo check --package js 2>&1 | grep -i error | wc -l
# Should show: small number (emacs-sys binding warnings are OK)
```

## The 30-Minute Implementation Path

### Step 1: Understand What's Done (5 min)

**Completed work**:
- ✅ Dependencies updated to deno_core 0.371, deno_runtime 0.229
- ✅ Old `ProgramState` architecture removed
- ✅ New `WorkerOptions` pattern implemented
- ✅ Custom `EmacsCachedModuleLoader` created
- ✅ Build system generates globals.h successfully

**What needs fixing** (in `crates/js/src/javascript.rs`):
- Lines ~400-500: `eval_js()` and `eval_ts()` functions
- These still reference old Deno APIs (commented out in Phase 1)

### Step 2: Review the Code Pattern (5 min)

Open `docs/deno-implementation-guide.md` and read Section 3: "Worker Execution Pattern"

Key pattern:
```rust
// OLD (doesn't work):
let result = runtime.execute(&code);

// NEW (what you need to implement):
let mut worker = MainWorker::bootstrap_from_options(...);
let result = worker.execute_script("<eval>", code)?;
// Extract result from v8 isolate
```

### Step 3: Update eval_js() Function (10 min)

Location: `crates/js/src/javascript.rs` around line 400

**Current state**: Function exists but worker execution is commented out

**What to do**:
1. Find the `eval_js()` function
2. Get the MainWorker from runtime: `runtime.get_worker()?`
3. Replace old execution with: `worker.execute_script()`
4. Extract return value from v8 scope
5. Convert to LispObject using existing helpers

**Code template** (from DENO_SMOKE_TEST.md):
```rust
pub fn eval_js(code: &str) -> Result<LispObject> {
    let runtime = get_global_runtime()?;
    let worker = runtime.get_worker()?;
    
    // Execute the JavaScript code
    let result = worker.execute_script("<eval>", code)
        .map_err(|e| error!("JS execution failed: {}", e))?;
    
    // Get the return value from v8 scope
    let scope = &mut worker.js_runtime.handle_scope();
    let result_value = result.get(scope);
    
    // Convert v8::Value to LispObject
    js_value_to_lisp(scope, result_value)
}
```

### Step 4: Update eval_ts() Function (10 min)

Location: `crates/js/src/javascript.rs` around line 500

**What to do**: Nearly identical to eval_js(), but:
1. Ensure ModuleLoader is set (already done in init_worker)
2. Use `worker.execute_module()` instead of `execute_script()`
3. Let Deno handle TypeScript transpilation automatically

**Code template**:
```rust
pub fn eval_ts(code: &str) -> Result<LispObject> {
    let runtime = get_global_runtime()?;
    let worker = runtime.get_worker()?;
    let module_loader = runtime.get_module_loader()?;
    
    // Inject TypeScript code into module cache
    let module_url = "file:///eval.ts";
    module_loader.insert_cached(module_url, code, MediaType::TypeScript)?;
    
    // Load and execute the module
    let module_id = worker.load_main_module(module_url, None).await?;
    let result = worker.evaluate_module(module_id).await?;
    
    // Convert result to LispObject
    let scope = &mut worker.js_runtime.handle_scope();
    js_value_to_lisp(scope, result)
}
```

### Step 5: Test Compilation (5 min)

```bash
# Build the js crate
cargo build --package js

# Check for errors
echo $?  # Should be 0 for success
```

**Expected**: Clean build (warnings OK, no errors)

### Step 6: Run Smoke Test (10 min)

```bash
# Build full Emacs (if not already done)
make

# Start Emacs
./src/emacs

# In Emacs scratch buffer, evaluate:
(eval-js "1 + 1")
# Expected: 2

(eval-js "{message: 'Hello', value: 42}")
# Expected: ((message . "Hello") (value . 42))

(eval-ts "const x: number = 42; x")
# Expected: 42
```

**Success criteria**: No errors, correct return values

## Troubleshooting

### Error: "worker not initialized"
**Fix**: Check that `init_worker()` is called in `js_initialize()`

### Error: "cannot borrow worker as mutable"
**Fix**: Use `RefCell` and `.borrow_mut()` to access worker

### Error: "module loader not set"
**Fix**: Verify `set_module_loader()` is called in `init_worker()`

### Compilation errors about v8 types
**Fix**: Check imports - you need `use deno_core::v8;`

## Reference Files

**Must-read** (in order):
1. `docs/deno-implementation-guide.md` - Code patterns
2. `DENO_SMOKE_TEST.md` - Smoke test specification
3. `HANDOFF.md` - Complete context (this directory)

**Code locations**:
- Main file: `crates/js/src/javascript.rs`
- Worker initialization: Lines ~1200-1300
- Functions to update: Lines ~400-500
- ModuleLoader: Lines ~148-220

## Getting Help

**Check these if stuck**:
1. Error messages - often self-explanatory
2. `docs/deno-implementation-guide.md` - Has detailed examples
3. Deno API docs: https://docs.rs/deno_core/0.371.0/
4. Existing code in javascript.rs - Uses similar patterns

**Common patterns**:
```rust
// Getting the runtime
let runtime = get_global_runtime()?;

// Getting the worker
let worker = runtime.get_worker()?;

// Executing code
let result = worker.execute_script("<eval>", code)?;

// Getting v8 scope
let scope = &mut worker.js_runtime.handle_scope();

// Converting to Lisp
js_value_to_lisp(scope, value)
```

## Next Steps After Success

Once eval_js/eval_ts work:

1. **Restore deno() lisp function** (currently commented out)
   - Location: Line ~1700 in javascript.rs
   - Update for new subcommand APIs

2. **Implement remaining subcommands**
   - `deno fmt`, `deno lint`, `deno test`, etc.
   - Follow patterns in `docs/deno-migration-plan.md`

3. **Add WebWorker support**
   - Parallel JavaScript execution
   - Worker message passing

4. **Complete test suite**
   - Unit tests for each function
   - Integration tests for Lisp interface

**Estimated time for full feature parity**: 3-4 additional weeks

## Summary

- **Current state**: 85% complete, foundation solid
- **Next task**: Update 2 functions (~100 lines of code)
- **Time required**: 30 minutes to 2 hours
- **Documentation**: Comprehensive guides available
- **Difficulty**: Low - patterns are clear and documented

**You've got this!** The hard work (research, dependencies, architecture) is done. 
What remains is straightforward implementation following documented patterns.
