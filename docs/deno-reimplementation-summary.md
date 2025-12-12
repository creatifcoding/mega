# Deno Support Reimplementation - Executive Summary

## Current State

The emacs-ng project previously had Deno JavaScript/TypeScript runtime support integrated, allowing users to:
- Execute JavaScript and TypeScript code within Emacs
- Call Lisp functions from JavaScript
- Call JavaScript functions from Lisp
- Use Deno's full standard library and ecosystem
- Run Deno CLI commands through Emacs

**This feature is currently disabled** because:
1. It depended on a custom fork of Deno that is now outdated
2. Deno's internal APIs have changed significantly
3. The code doesn't compile with current Rust/Deno versions
4. The repository mentions PR #463 working on bringing it back

## What Needs to Be Done

To reimplement Deno support for the latest version, we need to:

### 1. **Update Dependencies**
   - Replace custom Deno fork with official `deno_core` and `deno_runtime` crates
   - Current versions: deno_core 0.371.0, deno_runtime 0.229.0
   - Update rusty_v8 to 0.32.1

### 2. **Rewrite Worker Management**
   - Old: Used `ProgramState` and `create_main_worker` helper
   - New: Direct `MainWorker::bootstrap_from_options` with `WorkerOptions`
   - Simplify state management

### 3. **Implement Custom Module Loader**
   - Old: Used `program_state.file_fetcher.insert_cached()`
   - New: Implement `deno_core::ModuleLoader` trait
   - Handle module caching and resolution

### 4. **Update Permissions System**
   - Old: `Permissions::from_options(&flags.into())`
   - New: `PermissionsContainer` with different API
   - Maintain user-facing permission controls

### 5. **Rewrite Subcommands**
   - Update `eval_command`, `run_command`, `repl_command`, `test_command`
   - Adapt to new Deno CLI APIs
   - May require `deno_cli` crate for full functionality

### 6. **Fix TypeScript Compilation**
   - Modern Deno uses SWC via `deno_ast` crate
   - Need to integrate TypeScript transpilation
   - Maintain type-checking options

### 7. **Preserve V8 Bindings**
   - Lisp-to-JS function bindings (mostly unchanged)
   - JS-to-Lisp function calls (mostly unchanged)
   - Proxy object system (unchanged)
   - GC integration (unchanged)

## Implementation Approach

### Recommended: Hybrid Phased Approach

**Phase 1: Minimal Core** (2-3 weeks)
- Update dependencies
- Basic worker initialization
- Simple JS evaluation without modules
- Test lisp-JS interop basics

**Phase 2: Module Support** (2-3 weeks)
- Custom ModuleLoader implementation
- File system module loading
- Module caching
- TypeScript transpilation

**Phase 3: Advanced Features** (3-4 weeks)
- All subcommands (eval, run, test, repl)
- Full permissions control
- Inspector/debugger support
- Event loop integration

**Phase 4: Polish** (1-2 weeks)
- Documentation updates
- Migration guide
- Performance optimization
- Comprehensive testing

**Total Estimated Time: 8-12 weeks for full implementation**

## Key Technical Challenges

1. **API Incompatibility**: Deno's internal APIs changed fundamentally
   - **Risk**: High
   - **Mitigation**: Use official deno_runtime APIs, avoid internal dependencies

2. **Module Loading**: Need custom caching system
   - **Risk**: Medium
   - **Mitigation**: Well-documented ModuleLoader trait

3. **TypeScript Support**: Requires deno_ast integration
   - **Risk**: Medium
   - **Mitigation**: Use SWC through deno_ast crate

4. **REPL Implementation**: Complex interactive environment
   - **Risk**: High
   - **Mitigation**: May defer or use deno_cli as library

5. **State Management**: Thread-local complexity
   - **Risk**: Medium
   - **Mitigation**: Maintain existing architecture pattern

## Alternative Approaches

### Option A: Full Modern Rewrite (Recommended)
- **Pros**: Future-proof, maintainable, official APIs
- **Cons**: Time-consuming, requires deep Deno knowledge
- **Effort**: High (8-12 weeks)

### Option B: Fork Older Deno Version
- **Pros**: Faster initial implementation
- **Cons**: Security risks, no updates, technical debt
- **Effort**: Medium (4-6 weeks)

### Option C: Simplified JS-only Support
- **Pros**: Much simpler, fewer dependencies
- **Cons**: Loses TypeScript, loses Deno ecosystem
- **Effort**: Low (2-3 weeks)

### Option D: Use Deno as External Process
- **Pros**: No integration complexity
- **Cons**: Loses lisp-JS interop, performance overhead
- **Effort**: Low (1-2 weeks)

## Recommendation

**Proceed with Option A: Full Modern Rewrite using Hybrid Phased Approach**

### Justification:
1. **Sustainability**: Using official APIs ensures long-term maintainability
2. **Features**: Preserves all original functionality
3. **Community**: Easier for contributors to understand
4. **Performance**: Optimal integration with Emacs
5. **Security**: Benefits from Deno's security updates

### Success Criteria:
- ✅ Can evaluate JavaScript code from Lisp
- ✅ Can evaluate TypeScript code from Lisp
- ✅ Can call Lisp functions from JavaScript
- ✅ Can call JavaScript functions from Lisp
- ✅ Async/await works correctly
- ✅ Module imports work (file:// and https://)
- ✅ Basic Deno commands work (run, eval, fmt, test)
- ✅ Performance matches or exceeds old implementation
- ✅ Documentation is complete and accurate

## Files to Modify

Core implementation:
- `crates/js/Cargo.toml` - Update dependencies
- `crates/js/src/lib.rs` - Update exports
- `crates/js/src/javascript.rs` - Rewrite main runtime (1500+ lines)
- `crates/js/src/subcommands.rs` - Rewrite all subcommands (300+ lines)
- `crates/js/src/prelim.js` - Minimal changes

Documentation:
- `README.md` - Update feature status
- `docs/js/using-deno.md` - Update examples
- `docs/js/architecture.md` - Update architecture docs
- `docs/deno-migration-plan.md` - Migration notes (created)
- `docs/deno-implementation-guide.md` - Implementation guide (created)

Testing:
- Create `crates/js/tests/` - Integration tests
- Update CI configuration

## Next Steps for Implementation

1. **Set up development environment**
   - Clone repository
   - Install dependencies
   - Set up Rust nightly toolchain

2. **Start Phase 1: Minimal Core**
   - Update `Cargo.toml`
   - Create simplified worker initialization
   - Get basic `eval_js` working
   - Test with simple examples

3. **Iterate and test frequently**
   - Small incremental changes
   - Test each change
   - Commit often

4. **Document as you go**
   - Update docs with findings
   - Note any API differences
   - Create examples

## Conclusion

Reimplementing Deno support is a significant but achievable undertaking. The work requires:
- Deep understanding of both old and new Deno APIs
- Careful migration of existing functionality
- Comprehensive testing
- Updated documentation

This document, along with the migration plan and implementation guide, provides a complete roadmap for reimplementing Deno support in emacs-ng with modern, maintainable code using official Deno crates.

The estimated timeline of 8-12 weeks assumes:
- One experienced developer working full-time
- Familiarity with Rust, Deno, and V8
- Access to testing resources
- No major unexpected API changes

The result will be a robust, future-proof Deno integration that benefits from the latest Deno improvements while maintaining the unique Lisp-JavaScript interoperability that makes emacs-ng special.
