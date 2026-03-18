# AGENTS.md - Guidelines for AI Agents

## About This File

This file contains guidelines and biases that AI agents should follow when working on this codebase.

---

## General Principles

### 1. Code Quality Over Speed

- Always prefer clean, maintainable code over quick hacks
- If you need to write a hack, document WHY it's a hack and when it can be removed
- No "temporary" code - if it's worth writing, it's worth doing right

### 2. Test Everything

**This is non-negotiable.**

- Every function, every feature, every bug fix MUST have tests
- Tests should be written BEFORE or DURING implementation, not after
- If you touch a file, you own the tests for that file
- Test coverage should not decrease
- Run existing tests before making changes to ensure they still pass
- If tests don't exist for a feature you're modifying, write them

### 3. Refactor Continuously

- As you implement features, if you see code that can be improved, refactor it
- Don't leave "tech debt" for later - address it in the same PR
- Follow the boy scout rule: "Leave the code better than you found it"
- If refactoring touches multiple files, ensure all tests still pass

### 4. Type Safety

- Prefer strong typing over runtime checks
- Use Rust's type system to enforce invariants
- Avoid `unsafe` unless absolutely necessary - document why when used

---

## Rust Specific Guidelines

### Error Handling

```rust
// PREFERRED: Use Result types with meaningful errors
fn do_something() -> Result<Output, MyError> {
    // ...
}

// AVOID: Panicking for expected error cases
fn bad_example() -> Output {
    something.unwrap() // Don't do this
}
```

### Error Messages

- Errors should tell the user WHAT failed and WHY
- Include context: "Failed to X because Y"
- Log errors before returning them

```rust
// GOOD
fn load_config() -> Result<Config, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config at {:?}: {}", path, e))?;
    // ...
}
```

### Logging

- Use `println!` for startup/runtime info in development
- Use `eprintln!` for errors
- Include context in log messages

### Platform-Specific Code

- Use `#[cfg(target_os = "...")]` for platform-specific implementations
- Keep platform-specific code isolated in its own modules
- Document platform-specific behavior

```rust
// GOOD: Isolated platform code
#[cfg(target_os = "windows")]
mod platform {
    pub fn get_file_icon(path: &str) -> Result<Vec<u8>, String> { ... }
}

#[cfg(target_os = "macos")]
mod platform {
    pub fn get_file_icon(path: &str) -> Result<Vec<u8>, String> { ... }
}
```

### Dependencies

- Keep dependencies to a minimum
- Prefer well-maintained, popular crates
- Avoid crates with known security issues
- Document why each dependency is needed

---

## Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_works() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[test]
    fn test_error_handling() {
        // Test error cases
    }
}
```

### Test Naming

- Name tests descriptively: `test_feature_does_x`
- Use `#[should_panic]` for expected panics with `expected_message`
- Group related tests with common prefixes

### What to Test

- Happy path
- Edge cases
- Error conditions
- Boundary conditions (empty, max values, etc.)
- Platform-specific behavior

### Test Fixtures

- Create helper functions for common test setup
- Use temp directories for file operations
- Clean up after tests

---

## Frontend (TypeScript/React) Guidelines

### Type Safety

- Never use `any` - use proper types or `unknown`
- Define interfaces for all data structures
- Use TypeScript's strict mode

### Component Structure

```tsx
// PREFERRED: Small, focused components
interface Props {
  title: string;
  onSubmit: (data: FormData) => void;
}

export function MyComponent({ title, onSubmit }: Props) {
  // Component logic
}

// AVOID: Large monolithic components
export function BigComponent() {
  // 500 lines of code - split this up!
}
```

### State Management

- Use React hooks appropriately
- Keep state as local as possible
- Lift state up only when needed

### Error Handling

- Handle loading states
- Handle error states
- Show meaningful error messages to users

---

## File Organization

### Rust

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── commands/         # Tauri commands
│   ├── mod.rs
│   ├── file_ops.rs
│   └── ...
├── utils/            # Utility functions
│   ├── mod.rs
│   └── icons.rs
└── ...
```

### Frontend

```
src/
├── components/       # Reusable UI components
├── hooks/            # Custom React hooks
├── lib/              # Utility functions
├── pages/            # Page components
└── App.tsx           # Root component
```

---

## Git & PR Guidelines

### Commit Messages

- Use imperative mood: "Add feature" not "Added feature"
- First line: Short description (under 50 chars)
- Body: Explain WHAT and WHY, not HOW
- Reference issues: "Closes #123"

### PR Structure

- Keep PRs small and focused
- One feature or fix per PR
- Include tests in the same PR
- Update documentation if needed

---

## When Unsure

1. **Ask the user** - Don't assume
2. **Check existing code** - Follow patterns you see
3. **Check tests** - They show expected behavior
4. **Consider edge cases** - What could go wrong?

---

## Important Notes for This Project

### Building

- **DO NOT build the app** - The user will test and build it themselves
- Only run `cargo check` or `cargo build` to verify code compiles
- If you need to verify functionality, ask the user to test

### Cross-Platform Considerations

- Test on both Windows AND macOS
- Platform-specific code must be clearly marked
- Don't break Windows while adding macOS support (and vice versa)

### Tauri Specific

- Use Tauri APIs where possible instead of platform-specific code
- Remember frontend is separate process - IPC for communication
- Keep backend logic in Rust, frontend in TypeScript/React

### Performance

- App should be lightweight and fast
- Avoid unnecessary polling
- Use Tauri events for reactivity
- Lazy load heavy operations

---

## TL;DR

1. **Test everything** - No exceptions
2. **Refactor as you go** - Don't leave mess for later  
3. **Strong types** - No `any`, no `unwrap` without reason
4. **Meaningful errors** - Tell users what went wrong
5. **Ask when unsure** - Better to ask than assume
6. **Platform-aware** - Mark platform-specific code clearly
7. **Clean code** - Leave it better than you found it
