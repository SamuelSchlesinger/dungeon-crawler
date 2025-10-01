# Claude Development Workflow Guide

This document captures the development workflow and best practices discovered during the Bevy 0.8 → 0.17 migration of this dungeon crawler game.

## Table of Contents
1. [Initial Assessment & Planning](#initial-assessment--planning)
2. [Dependency Management](#dependency-management)
3. [Incremental Migration Strategy](#incremental-migration-strategy)
4. [Code Review & Bug Detection](#code-review--bug-detection)
5. [Testing & Validation](#testing--validation)
6. [Git Workflow](#git-workflow)
7. [Key Lessons Learned](#key-lessons-learned)

---

## Initial Assessment & Planning

### 1. Research Phase
**ALWAYS start by researching the latest information:**
- Use web search to find the latest version information (don't rely on outdated training data)
- Fetch migration guides from official sources
- Check GitHub releases and release notes
- Look at example code in the official repository

**Example from this project:**
```bash
# Step 1: Search for latest version
WebSearch: "Bevy engine latest version 2025"

# Step 2: Fetch migration guide
WebFetch: "https://bevy.org/learn/migration-guides/0-16-to-0-17/"

# Step 3: Clone examples for reference
git clone --depth 1 --branch v0.16.0 https://github.com/bevyengine/bevy.git /tmp/bevy-0.16
```

### 2. Create a Task List
**Use TodoWrite to track all major steps:**
```
1. Update Cargo.toml dependencies
2. Update submodule dependencies
3. Build and identify compilation errors
4. Fix API changes systematically
5. Run code review
6. Fix bugs
7. Test thoroughly
8. Commit changes
```

**Benefits:**
- Provides visibility to the user
- Prevents forgetting steps
- Allows tracking progress
- Helps prioritize work

---

## Dependency Management

### 1. Updating Dependencies
**Order matters - update in this sequence:**

```toml
# Step 1: Update main Cargo.toml
bevy = "0.17"

# Step 2: Update submodule Cargo.toml files
# (positioning/Cargo.toml in this project)
bevy = { version = "0.17", optional = true }

# Step 3: Let cargo update transitive dependencies
cargo update
```

### 2. Handling Submodules
**When using Git submodules:**

1. **Add as submodule with SSH:**
   ```bash
   git submodule add git@github.com:user/repo.git path/to/submodule
   ```

2. **Update submodule code first:**
   ```bash
   cd submodule
   # Make changes
   git add -A
   git commit -m "Update to new version"
   git push origin branch
   cd ..
   ```

3. **Then update parent repo:**
   ```bash
   git add submodule
   git commit -m "Update submodule reference"
   ```

### 3. Dependency Update Strategy
**Use incremental updates for major version jumps:**

- For 8 version jump (0.8 → 0.16): Update to intermediate stable version first
- For 1 version jump (0.16 → 0.17): Direct update is fine
- Always check if the update "just works" before making code changes

**In this project:**
- 0.8 → 0.16 required extensive API changes
- 0.16 → 0.17 compiled with no code changes (pleasant surprise!)

---

## Incremental Migration Strategy

### 1. Build Early, Build Often
**Compile frequently to catch issues early:**

```bash
# After each major change category:
cargo build 2>&1 | head -100    # See first errors
cargo build 2>&1 | tail -100    # See final errors
cargo build 2>&1 | grep "error\[" -A 5  # See detailed errors
```

### 2. Fix Errors by Category
**Group similar fixes together:**

**Category 1: State Management**
```rust
// OLD (Bevy 0.8)
.add_state(GameState::Menu)
ResMut<State<GameState>>
state.set(GameState::Playing).unwrap()

// NEW (Bevy 0.16+)
.init_state::<GameState>()
ResMut<NextState<GameState>>
next_state.set(GameState::Playing)
```

**Category 2: System Scheduling**
```rust
// OLD (Bevy 0.8)
.add_system(system_fn)
.add_system_set(SystemSet::new()
    .with_run_criteria(FixedTimestep::steps_per_second(30.))
    .with_system(system_fn))

// NEW (Bevy 0.16+)
.add_systems(Update, system_fn)
.insert_resource(Time::<Fixed>::from_hz(30.0))
.add_systems(FixedUpdate, system_fn)
```

**Category 3: Bundles → Components**
```rust
// OLD (Bevy 0.8)
commands.spawn_bundle(SpriteSheetBundle {
    sprite: TextureAtlasSprite::new(index),
    texture_atlas: atlas_handle,
    transform: Transform::default(),
    ..default()
})

// NEW (Bevy 0.16+)
commands.spawn((
    Sprite::from_atlas_image(
        texture_handle,
        TextureAtlas { layout: layout_handle, index }
    ),
    Transform::default(),
    Visibility::default(),
))
```

**Category 4: Input System**
```rust
// OLD (Bevy 0.8)
keyboard_input: Res<Input<KeyCode>>
if keyboard_input.pressed(KeyCode::W)

// NEW (Bevy 0.16+)
keyboard_input: Res<ButtonInput<KeyCode>>
if keyboard_input.pressed(KeyCode::KeyW)
```

### 3. Use Specialized Agents for Complex Tasks
**For systematic refactoring, delegate to a specialized agent:**

```rust
Task {
    subagent_type: "general-purpose",
    description: "Fix all Bevy 0.16 API changes",
    prompt: "
        You are helping migrate... [detailed instructions]

        Key API changes:
        1. Bundle changes: ...
        2. TextureAtlas changes: ...
        [etc]

        Work through each file systematically:
        1. Start with setup files
        2. Then fix system files
        3. Run cargo build frequently

        Return a summary of changes made.
    "
}
```

**Benefits:**
- Agent can work autonomously through many files
- You get a detailed report back
- Reduces token usage in main conversation
- Can run in parallel with other work

---

## Code Review & Bug Detection

### 1. Always Do a Post-Migration Code Review
**Even if the code compiles, review for logic errors:**

Use a specialized agent:
```rust
Task {
    subagent_type: "general-purpose",
    description: "Code review and compatibility check",
    prompt: "
        Conduct a comprehensive code review of this Bevy 0.17 game.

        1. Code Review - Check for:
           - API usage correctness
           - Potential bugs from migration
           - System ordering issues
           - Resource management issues

        2. Asset Integration - Verify:
           - Asset loading patterns
           - TextureAtlas configuration
           - Sprite indices

        3. System Compatibility - Check:
           - System parameters
           - State transitions
           - Input handling

        Be thorough and flag anything suspicious.
    "
}
```

### 2. Common Bug Patterns to Check

**Coordinate Confusion:**
```rust
// BUG: Using wrong coordinate
Transform::from_xyz(
    position.z as f32,  // ❌ Should be x
    position.z as f32,  // ❌ Should be y
    0.0
)

// FIX:
Transform::from_xyz(
    position.x as f32,  // ✓
    position.y as f32,  // ✓
    0.0
)
```

**Logic Order Issues:**
```rust
// BUG: Unreachable condition
if fraction <= 0.5 {
    // yellow
} else if fraction <= 0.2 {  // ❌ Never reached!
    // red
}

// FIX:
if fraction <= 0.2 {
    // red
} else if fraction <= 0.5 {
    // yellow
}
```

**Resource Synchronization:**
```rust
// BUG: Entity despawned but still in resource
commands.entity(enemy).despawn();
// enemies resource still tracks this entity! ❌

// FIX: Add cleanup system
fn cleanup_dead_enemies(
    mut enemies: ResMut<Enemies>,
    query: Query<(Entity, &Position), With<Enemy>>,
) {
    let mut new_enemies = Enemies::new();
    for (entity, position) in query.iter() {
        new_enemies.insert(*position, entity);
    }
    *enemies = new_enemies;
}
```

### 3. Review System Ordering
**Check that systems run in the right order:**

```rust
// Combat, then cleanup, then victory check
.add_systems(
    FixedUpdate,
    (
        combat,                      // May despawn enemies
        cleanup_dead_enemies,        // Sync resource with reality
        victory,                     // Check if all enemies dead
    ).run_if(in_state(GameState::Playing))
)
```

**Red flags:**
- Systems that modify same resources without ordering
- Cleanup systems that run before the systems that create work
- Victory/defeat checks before combat resolution

---

## Testing & Validation

### 1. Multi-Configuration Testing
**Test both debug and release builds:**

```bash
# Debug build (faster compilation, slower runtime)
cargo build
# Check: Compiles without errors

# Release build (slower compilation, faster runtime, more optimizations)
cargo build --release
# Check: Compiles without errors, no optimization-related issues
```

### 2. Check for Deprecation Warnings
```bash
cargo build 2>&1 | grep -i "deprecated"
```

**Fix immediately:**
```rust
// DEPRECATED in Bevy 0.17
EventReader<T>

// REPLACEMENT
MessageReader<T>
```

### 3. Validate Asset Paths
**Verify assets exist and are correctly referenced:**

```rust
// Check in code review:
asset_server.load("fonts/FreeMono.ttf")  // Does this file exist?
asset_server.load("tiles.png")           // At correct path?

// For TextureAtlas:
TextureAtlasLayout::from_grid(
    UVec2::new(32, 32),  // Tile size
    64, 48,              // Grid: 64x48 = 3072 tiles
    None, None
)
// Verify: tiles.png is 2048x1536 pixels (64*32 x 48*32)
```

### 4. Runtime Testing Checklist
After migration compiles:
- [ ] Game launches without panics
- [ ] Player spawns at correct location
- [ ] Camera follows player correctly
- [ ] Input responds correctly
- [ ] Sprites render at correct positions
- [ ] UI displays properly
- [ ] State transitions work
- [ ] Combat/collision works
- [ ] No resource desync issues

---

## Git Workflow

### 1. Commit Strategy for Large Migrations
**For multi-step migrations, use one commit per major version:**

```bash
# Option A: One commit for entire migration (used in this project)
git add -A
git commit -m "Migrate from Bevy 0.8 to 0.17 with bug fixes

## Migration Changes
[detailed list]

## Bug Fixes
[detailed list]

## New Features
[detailed list]
"

# Option B: Multiple commits (for even larger migrations)
git commit -m "Migrate from Bevy 0.8 to 0.16"
git commit -m "Migrate from Bevy 0.16 to 0.17"
git commit -m "Fix coordinate bugs discovered in review"
```

### 2. Submodule Commit Sequence
**ALWAYS commit submodules before the parent:**

```bash
# 1. Commit submodule first
cd positioning
git add -A
git commit -m "Update to Bevy 0.17"
git push origin dev

# 2. Then commit parent
cd ..
git add positioning  # Stage the submodule reference update
git add [other files]
git commit -m "Update to Bevy 0.17 (includes positioning submodule update)"
git push origin main
```

### 3. Comprehensive Commit Messages
**Structure for migration commits:**

```markdown
# Title: What was done
Migrate from Bevy 0.8 to 0.17 with bug fixes

## Migration Changes
- List API changes made
- Grouped by category
- Be specific about versions

## Bug Fixes
- List bugs found and fixed
- Include file locations
- Explain the impact

## New Features
- Any new systems/functionality added
- Performance improvements
- Architecture changes

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Benefits:**
- Future developers understand what happened
- Easy to review changes
- Good documentation trail
- Searchable history

### 4. SSH vs HTTPS for Git
**Prefer SSH for repositories you own:**

```bash
# Change existing remote to SSH
git remote set-url origin git@github.com:user/repo.git

# Add submodule with SSH
git submodule add git@github.com:user/subrepo.git path
```

**Benefits:**
- No password prompts
- Works with SSH keys
- More secure
- Better for automation

---

## Key Lessons Learned

### 1. Migration Philosophy
**Progressive Elaboration:**
- Start with big picture (update dependencies)
- Compile to find issues
- Fix issues by category
- Review for bugs
- Test thoroughly

**Don't Assume Anything Works:**
- Even if code compiles, review logic
- Check coordinates carefully (x/y/z confusion is common)
- Verify resource synchronization
- Test state transitions

### 2. When to Use Agents vs Direct Work

**Use Agents For:**
- Systematic refactoring across many files
- Code review of entire codebase
- Research and information gathering
- Repetitive changes

**Do Directly For:**
- Small targeted fixes
- Architectural decisions
- Debugging specific issues
- Final validation

### 3. Communication with User

**Be Concise:**
- User asked for compact output
- Summarize rather than explain every detail
- Use checkmarks and status indicators
- Only show critical information

**Be Transparent:**
- Show what you're doing
- Explain major decisions
- Flag risks and concerns
- Ask when uncertain

**Track Progress:**
- Use TodoWrite for visibility
- Update todos as you progress
- Mark completion clearly

### 4. Tool Usage Patterns

**Read Before Edit:**
- Always read files before editing
- Understand context and structure
- Check for similar patterns in codebase

**Batch Similar Changes:**
- Group related files together
- Use single edits for multiple issues when possible
- Test after each batch

**Use Grep and Glob Effectively:**
```bash
# Find all files with pattern
Glob: "**/*.rs"

# Search for specific API usage
Grep: pattern="EventReader", output_mode="files_with_matches"

# Then read and fix each file
```

### 5. Common Pitfalls in Game Engine Migration

**Problem: Copy-Paste Errors**
```rust
// Easy to copy wrong variable:
x = position.x  // ✓
y = position.x  // ❌ Should be position.y
```
**Solution:** In code review, look for duplicate coordinate usage

**Problem: Resource Desynchronization**
```rust
// Entity despawned but still in custom resource
```
**Solution:** Add cleanup systems that run after despawn-causing systems

**Problem: Conditional Logic Errors**
```rust
// Ordering matters for overlapping conditions
if x <= 50 { }      // This catches x <= 20 too!
else if x <= 20 { } // Never reached
```
**Solution:** Review all conditional chains in code review

**Problem: Deprecated API Usage**
```rust
// Compiles but deprecated
EventReader<T>
```
**Solution:** Check for deprecation warnings even if code compiles

### 6. Documentation Strategy

**Document as You Go:**
- Create Claude.md with workflow insights
- Note decisions and rationale
- Record common patterns
- List gotchas and solutions

**Structure Documentation:**
- Table of contents for navigation
- Clear sections for different topics
- Code examples for clarity
- Checklists for processes

**Keep It Actionable:**
- Focus on "how to" not just "what"
- Include specific commands and code
- Provide templates and patterns
- Explain the "why" behind decisions

---

## Quick Reference Checklist

### Starting a Migration
- [ ] Research latest version online
- [ ] Fetch migration guide
- [ ] Create task list with TodoWrite
- [ ] Clone examples repo for reference
- [ ] Check for breaking changes list

### During Migration
- [ ] Update dependencies in order (main → submodules)
- [ ] Build frequently to catch errors early
- [ ] Fix errors by category
- [ ] Use agents for systematic refactoring
- [ ] Update todos as you progress

### After Migration
- [ ] Run comprehensive code review
- [ ] Fix any bugs discovered
- [ ] Test debug and release builds
- [ ] Check for deprecation warnings
- [ ] Validate asset paths and configurations

### Before Committing
- [ ] All builds pass
- [ ] No critical warnings
- [ ] Tests run successfully (if applicable)
- [ ] Submodules committed first
- [ ] Comprehensive commit message written
- [ ] Changes pushed to remote

---

## Conclusion

This workflow successfully migrated a Bevy game through 9 major versions (0.8 → 0.17) while discovering and fixing 7 critical bugs. The key principles:

1. **Research first** - Don't rely on outdated information
2. **Plan systematically** - Use task lists and agents
3. **Build incrementally** - Catch issues early
4. **Review thoroughly** - Compilation isn't correctness
5. **Test comprehensively** - Multiple configurations
6. **Document clearly** - Help future developers

By following this workflow, complex migrations become manageable, bugs are caught early, and the final result is robust and maintainable.

---

## Project Management Methodology

### Overview
This section describes the project management system for tracking multi-feature development work across Claude Code sessions. The goal is to maintain clear progress visibility and ensure work propagates correctly between sessions.

### Project File Structure

**Location**: `/home/samuel/Code/dungeon-crawler/PROJECT.md`

This file tracks:
1. Feature backlog (features not yet started)
2. In-progress features (currently being implemented)
3. Completed features (finished and tested)
4. Known issues and bugs
5. Technical debt items

### Project.md Format

```markdown
# Dungeon Crawler - Project Status

Last Updated: [Date]
Session: [Session identifier]

## In Progress
- [ ] Feature Name
  - Status: [Percentage complete or phase]
  - Files Modified: [List of files]
  - Remaining Work: [What's left]
  - Blockers: [Any blockers]

## Backlog
- [ ] Feature Name
  - Priority: High/Medium/Low
  - Description: [Brief description]
  - Dependencies: [Other features needed first]
  - Estimated Complexity: Small/Medium/Large

## Completed
- [x] Feature Name (Completed: [Date])
  - Files Modified: [List]
  - Commits: [Git commit references]

## Known Issues
- Issue description
  - Severity: Critical/High/Medium/Low
  - Files: [Affected files]

## Technical Debt
- Debt item
  - Impact: [How it affects development]
  - Effort to Fix: [Estimated effort]
```

### TodoWrite Usage Strategy

**When to Use TodoWrite:**
1. **Session Start**: Create todos for all planned work from PROJECT.md "In Progress" section
2. **During Work**: Update todo status in real-time as you complete each task
3. **Session End**: Clear todos and update PROJECT.md with final status

**TodoWrite vs PROJECT.md:**
- **TodoWrite**: Short-term, within-session task tracking (doesn't persist between sessions)
- **PROJECT.md**: Long-term feature and project tracking (persists between sessions)

**Pattern:**
```
Session Start:
1. Read PROJECT.md
2. Create TodoWrite items for current session work
3. Work through todos, marking in_progress → completed

Session End:
1. Update PROJECT.md with progress
2. Clear or complete all todos
3. Document any blockers or issues
```

### Multi-Feature Development Workflow

**Phase 1: Planning**
1. User requests multiple features
2. Create comprehensive feature list
3. Write all features to PROJECT.md backlog
4. Prioritize and identify dependencies
5. Move first feature(s) to "In Progress"

**Phase 2: Implementation (Iterative)**
1. Read PROJECT.md to see current in-progress features
2. Create TodoWrite tasks for implementation steps
3. Implement feature systematically:
   - Read relevant files
   - Design component/system architecture
   - Implement core functionality
   - Test and validate
   - Mark todo as completed
4. Update PROJECT.md with progress
5. Move to next feature

**Phase 3: Integration**
1. After all features implemented, test interactions
2. Fix integration bugs
3. Run comprehensive testing
4. Update PROJECT.md with completion status

**Phase 4: Finalization**
1. Build both debug and release
2. Create comprehensive commit
3. Push to GitHub
4. Move all features to "Completed" in PROJECT.md

### Feature Implementation Pattern

For each feature:

```rust
// 1. Define new components/resources if needed
// File: src/components.rs or src/resources.rs

// 2. Create system implementation
// File: src/systems/feature_name.rs

// 3. Add system to schedule
// File: src/main.rs

// 4. Test the system
// Build and run game

// 5. Document in PROJECT.md
```

### Session Continuity Strategy

**Between Sessions:**
1. PROJECT.md is the source of truth
2. Git commits preserve code state
3. Claude.md preserves methodology

**Starting New Session:**
```
1. Read PROJECT.md to understand current state
2. Read last commits to see recent changes
3. Build project to ensure clean state
4. Create TodoWrite list for session work
5. Begin implementation
```

**Ending Session:**
```
1. Update PROJECT.md with all progress
2. Commit meaningful work (even if incomplete)
3. Document blockers or next steps in PROJECT.md
4. Clear or complete all todos
```

### Handling Large Feature Sets

**Break Down Strategy:**
1. Divide large features into sub-features
2. Implement in logical order (dependencies first)
3. Test after each sub-feature
4. Keep PROJECT.md updated with granular progress

**Example Breakdown:**
```
Feature: Enemy Variety
├── Sub-feature 1: Add EnemyType enum and component
├── Sub-feature 2: Create stat variations
├── Sub-feature 3: Add sprite mapping
├── Sub-feature 4: Update spawn logic
└── Sub-feature 5: Test different enemy types
```

### Preventing Context Loss

**Key Files to Track:**
- `PROJECT.md` - Project status
- `Claude.md` - Methodology and workflows
- Recent git commits - Code changes
- `Cargo.toml` - Dependencies

**Pattern for Complex Work:**
1. Make small, frequent commits
2. Update PROJECT.md after each feature
3. Reference file:line in PROJECT.md for clarity
4. Document design decisions in comments

### Testing Strategy

**After Each Feature:**
- Compile successfully
- Run game and test feature in isolation
- Check for visual/audio feedback
- Verify no regressions

**Before Commit:**
- Test all features together
- Check debug and release builds
- Verify git status is clean
- Update PROJECT.md to reflect completion

### Communication with User

**During Implementation:**
- Brief status updates after each major feature
- Flag blockers immediately
- Ask for clarification when needed
- Show progress via TodoWrite

**Session Completion:**
- Summarize what was completed
- Note what remains (if anything)
- Highlight any issues or concerns
- Confirm PROJECT.md reflects current state

---

*Generated during the Bevy 0.8 → 0.17 migration of dungeon-crawler*
*Updated with Project Management Methodology: 2025-10-01*
*Claude Code Version: Sonnet 4.5*
