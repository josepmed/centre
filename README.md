# Centre 🌱

A calm, terminal-based daily rhythm companion with context-aware time tracking and visual growth metaphors.

Centre helps you manage your entire day with a gentle, intentional approach to time management. Track tasks and subtasks, switch between life contexts (work, lunch, gym, personal), monitor elapsed time vs estimates, and watch your focus garden grow throughout the day.

Centre supports both global task tracking (`~/.centre/`) and local project-specific tasks (`.centre/` directory).

## Features

- **Context mode switching**: Track your daily rhythm across 6 modes (💼 Working, 🍽 Lunch, 🏋️ Gym, 🍲 Dinner, 🏡 Personal, 🌙 Sleep)
- **Dynamic TUI layout**: Today's Centre List, Details Pane, Focus Garden, and optional Done Tasks view
- **Hierarchical tasks**: Support for tasks with nested subtasks (one level deep)
- **Smart estimates**: Shows subtask-based estimates alongside task estimates when they differ
- **Real-time tracking**: Independent timers for tasks and subtasks with parallel running support
- **Intelligent mode handling**: Tasks automatically pause in non-working modes and can't be started until you return to Working mode
- **Visual metaphors**: Plant emojis (🌱🌿🌵) that evolve based on your progress
- **Soft estimates**: Gentle prompts when time estimates are reached, not hard limits
- **Tags**: Optional tags for categorization with visual badges (e.g., [urgent] [bug])
- **Daily file system**: Each day gets its own file (YYYY-MM-DD.md) with ACTIVE, DONE, and ARCHIVED sections
- **Automatic migration**: Tasks automatically carry forward to new days
- **Daily reports**: Comprehensive statistics reports with context mode breakdowns generated automatically at day transitions
- **Done tasks view**: Toggle view of completed tasks from today (press `c`)
- **Task reordering**: Use Shift+↑/↓ to reorganize tasks and subtasks
- **Archive system**: Archive tasks you want to keep but not show in active list
- **Journal**: Built-in daily journal with cursor support and word navigation
- **Human-friendly persistence**: Plain Markdown files you can edit by hand
- **Local & Global modes**: Use global `~/.centre/` or local `.centre/` for project-specific tasks

## Installation

### Prerequisites

- Rust toolchain (1.70 or newer)
- A terminal that supports UTF-8 and emojis

### Build from source

```bash
cargo build --release
```

The binary will be available at `target/release/centre`.

### Install

```bash
cargo install --path .
```

## Quick Start

### Global Mode (Default)

```bash
# Run Centre
centre
```

On first run, Centre creates `~/.centre/` with daily files:
- `YYYY-MM-DD.md` - Daily file with ACTIVE, DONE, and ARCHIVED sections
- `journal-YYYY-MM-DD.md` - Daily journal entries
- `report-YYYY-MM-DD.md` - Automatically generated daily statistics reports
- `meta.json` - App metadata including current mode and mode time tracking
- `archive.md` - Long-term archived tasks
- `done.log.md` - Legacy done log (deprecated)

### Local Mode (Project-specific)

```bash
# Initialize a local .centre directory for project-specific tasks
centre init

# Run centre from anywhere in the project
cd myproject/src/components
centre  # Will use myproject/.centre directory
```

Centre searches for a `.centre` directory by walking up from your current directory. If found, it uses that directory; otherwise it falls back to the global `~/.centre` directory.

## CLI Commands

Centre provides several commands beyond the default TUI mode:

### Report Generation

Generate comprehensive daily statistics reports:

```bash
# Generate report for today (default)
centre report

# Generate report for a specific date
centre report --date 2025-11-10

# Generate report with custom output path
centre report --output ~/Documents/my-report.md

# Combine flags
centre report --date 2025-11-10 --output /tmp/yesterday-report.md
```

**Report Contents:**
- Summary (task counts, total time, efficiency, completion rate)
- Context Modes (time spent in each mode: Working, Lunch, Gym, Dinner, Personal, Sleep)
- Time & Productivity (running/paused/idle time, sessions, interruptions)
- Estimation Accuracy (over/under estimates, accuracy percentage)
- Task Completion (completed count, average time, fastest/longest tasks)
- Tag Analysis (performance breakdown by tag)
- Tasks Breakdown (detailed list with subtasks and metrics)

**Automatic Report Generation:**
Reports are automatically generated in two scenarios:
1. **Day transition while app is running**: When midnight passes, a report for the day that just ended is saved
2. **App startup on new day**: When you start Centre and today's file doesn't exist, a report for yesterday is generated

All automatic reports are saved to `~/.centre/report-YYYY-MM-DD.md` or `.centre/report-YYYY-MM-DD.md` for local mode.

### Initialize Local Directory

Create a project-specific `.centre` directory:

```bash
centre init
```

This creates a `.centre/` directory in your current location for project-specific task tracking.

## Keybindings

### Navigation
- `↑` / `↓` - Move selection up/down
- `Shift+↑` / `Shift+↓` - Reorder task/subtask (move up/down in list)
- `Space` - Collapse/expand subtasks
- `c` - Toggle done tasks view (show/hide completed tasks from today)

### Task Management
- `Enter` - Toggle run/pause for selected task (only works in Working mode)
- `+` / `-` - Increase/decrease estimate (default: 15 min increments)
- `d` - Mark task as done
- `p` - Postpone task to tomorrow
- `r` - Archive task/subtask (removes from view, saves to archive.md)
- `x` / `Delete` - Archive selected task or subtask
- `n` - Edit notes (opens external $EDITOR)
- `a` - Add new task (opens input form)
- `A` - Add subtask to selected task (opens input form)
- `j` - Toggle journal editing mode
- `m` - Open context mode selector (Working, Lunch, Gym, Dinner, Personal, Sleep)
- `q` - Quit (autosaves)

### Input Form (Adding Task/Subtask)
When adding a new task or subtask:
- Type to enter text in the current field (title, notes, or tags)
- `Tab` - Switch between fields (title → notes → tags → title)
- `Backspace` - Delete last character
- `Enter` - Create task/subtask (default estimate: 1.0h)
- `Esc` - Cancel without creating
- Tags should be comma-separated (e.g., "urgent, bug, frontend")

### Context Mode Selector (press `m`)
Select your current life context:
- `1` - 💼 Working (timers run normally)
- `2` - 🍽 Lunch (all tasks paused)
- `3` - 🏋️ Gym (all tasks paused)
- `4` - 🍲 Dinner (all tasks paused)
- `5` - 🏡 Personal (all tasks paused)
- `6` - 🌙 Sleep (all tasks paused)
- `Esc` - Cancel

### Modal (Estimate Reached)
When a running task reaches its estimate:
- `d` - Mark as done
- `e` - Extend estimate (+30 minutes)
- `s` / `p` - Pause
- `t` - Postpone to tomorrow
- `Esc` - Close modal

### Journal Editing
When in journal editing mode (press `j` to enter):
- Type normally to edit journal text
- `←` / `→` - Move cursor character by character
- `Home` / `End` - Jump to start/end of journal
- `Option+←` / `Option+→` - Jump backward/forward by word (Mac)
- `Enter` - Insert newline
- `Backspace` / `Delete` - Remove characters
- `Esc` - Exit journal editing mode

## Context Modes

Centre helps you track your entire day, not just work time. The context mode system lets you mark what you're doing throughout the day, providing a complete picture of your daily rhythm.

### How It Works

Press `m` to open the mode selector and choose your current context:

- **💼 Working** - Active focus time (default mode). Timers run normally, tasks can be started/paused/resumed.
- **🍽 Lunch** - Break time for meals. All running tasks automatically pause, new tasks cannot be started.
- **🏋️ Gym** - Exercise time. All running tasks automatically pause.
- **🍲 Dinner** - Evening meal time. All running tasks automatically pause.
- **🏡 Personal** - Personal errands and non-work activities. All running tasks automatically pause.
- **🌙 Sleep** - Night/rest mode. All running tasks automatically pause.

### Behavior

**When you switch modes:**
- Switching from Working → any other mode: All running tasks automatically pause
- Switching to Working mode: Previously paused tasks automatically resume
- Your current mode is displayed in the header: "Today's Centre 🌱 (Date) — 💼 Working"

**Task control by mode:**
- **Working mode**: Press Enter to start/pause/resume tasks normally
- **Non-working modes**: Press Enter only pauses running tasks, cannot start or resume tasks

### Time Tracking

Centre tracks time spent in each mode throughout your day:
- **Focus Garden** displays mode times: `💼 Working 5h 30m | 🍽 Lunch 45m | 🏋️ Gym 30m`
- **Daily reports** include a Context Modes section showing time distribution
- **Persistence**: Mode times are saved and accumulate across app sessions
- **Daily reset**: Mode times reset automatically when a new day starts

### Contextual Phrases

When in non-working modes, the Focus Garden displays contextual encouragement:
- 🍽 Lunch: "Nourish before you bloom again"
- 🏋️ Gym: "Strength feeds focus"
- 🍲 Dinner: "Evening nourishment"
- 🏡 Personal: "Tending your own garden"
- 🌙 Sleep: "Rest — tomorrow's seeds await"

## File Format

Centre uses plain Markdown files that you can edit directly.

### Daily File (YYYY-MM-DD.md)

Each day has its own file with three sections:

```markdown
# 2025-11-11

## ACTIVE

- [RUNNING] Write project proposal
  est: 2.0h
  elapsed: 1.3h
  created: 2025-11-11T09:00:00
  notes: |
    finalize argument for timeline
  tags: urgent, writing
  state_history:
    - 2025-11-11T09:00:00: None -> Idle
    - 2025-11-11T10:00:00: Idle -> Running
  subtasks:
    - [PAUSED] Outline sections
      est: 1.0h
      elapsed: 0.7h
      created: 2025-11-11T09:05:00
      notes: |
        bullet the main points
      tags: research
      state_history:
        - 2025-11-11T09:05:00: None -> Idle
        - 2025-11-11T10:00:00: Idle -> Running
        - 2025-11-11T10:30:00: Running -> Paused

- [IDLE] Refactor centre code
  est: 1.5h
  elapsed: 0.0h
  created: 2025-11-11T09:30:00
  notes: |
    clean up state mgmt
  tags: refactor, code

## DONE

- [DONE] Morning standup
  est: 0.25h
  elapsed: 0.20h
  created: 2025-11-11T09:00:00
  completed: 2025-11-11T09:20:00
  tags: meeting
  state_history:
    - 2025-11-11T09:00:00: None -> Idle
    - 2025-11-11T09:00:00: Idle -> Running
    - 2025-11-11T09:20:00: Running -> Done

### Analytics
- **Calendar Time**: 0.33h (from creation to completion)
- **Active Time**: 0.20h (time in RUNNING state)
- **Interruptions**: 0
- **Sessions**: 1

## ARCHIVED

- [IDLE] Old task that's no longer relevant
  est: 1.0h
  elapsed: 0.0h
  created: 2025-11-10T15:00:00
```

**Status tags**: `IDLE`, `RUNNING`, `PAUSED`, `DONE`
**Time format**: Hours with decimals (e.g., `1.25h` = 1 hour 15 minutes)
**Timestamps**: ISO 8601 format (YYYY-MM-DDTHH:MM:SS)

**Task Migration**: When a new day starts, incomplete tasks from the ACTIVE section are automatically copied to the new day's file.

### Report File (report-YYYY-MM-DD.md)

Comprehensive daily statistics in Markdown format (see CLI Commands section for details).

## Workflow

### Morning
1. Launch Centre (starts in Working mode 💼)
2. If it's a new day, incomplete tasks from yesterday are automatically carried forward
3. A report for the previous day is automatically generated
4. Add or adjust tasks for the day
5. Use the journal (`j` key) to note your intentions or plan
6. Start your first task with `Enter`

### During the day
7. Switch context modes with `m` as your day flows (Working → Lunch → Gym → Working)
8. Tasks automatically pause when leaving Working mode, resume when you return
9. When an estimate is reached, choose what to do next (Done, Extend, Pause, or Postpone)
10. Run multiple tasks in parallel if needed (tasks and subtasks track independently)
11. Add notes to track context with `n`
12. Update journal throughout the day to capture insights
13. Monitor your Focus Garden to see overall progress and mode time breakdown

### End of day
14. Mark completed tasks as done with `d`
15. Postpone unfinished work with `p` (moves to tomorrow's file)
16. Archive tasks that are no longer relevant with `r` or `x`
17. Review your journal, Focus Garden stats, and mode time distribution
18. Switch to Sleep mode (🌙) if desired to track rest time
19. Quit with `q` - everything autosaves
20. If the app runs past midnight, it will automatically:
    - Generate a report for the day that just ended (including mode times)
    - Show a modal requiring restart for the new day

## Configuration

Centre uses sensible defaults. Configuration file support is planned for v1.1.

Default settings:
- **Tick rate**: 250ms
- **Estimate step**: 15 minutes
- **Global directory**: `~/.centre/`
- **Local directory**: `.centre/` (when using `centre init`)
- **Emoji enabled**: Yes (falls back to ASCII: `*`, `+`, `!`)

### Environment Variables

- `$EDITOR` - External editor for notes (default: `vi` on Unix, `notepad` on Windows)

## Architecture

```
centre/
├── src/
│   ├── main.rs              # Entry point, CLI parsing, event loop
│   ├── app.rs               # AppState, core mutations, business logic, mode management
│   ├── domain/              # Domain models (Item, TimeTracking, StateEvent, GlobalMode)
│   ├── persistence/         # Markdown parser/serializer, migration, file management
│   │   ├── metadata.rs      # JSON metadata (mode tracking, app state)
│   │   └── ...
│   ├── report/              # Statistics calculation and report generation
│   │   ├── stats.rs         # Statistics aggregation (global, tag, estimation)
│   │   └── generator.rs     # Markdown report generation with mode stats
│   ├── ui/                  # Ratatui rendering (list, details, garden, journal panes, modals)
│   ├── input/               # Keybinding handler for all UI modes
│   └── ticker.rs            # Timer tick logic
```

**Key design decisions**:
- Pure domain logic separated from UI and persistence
- Context-aware time tracking across 6 daily life modes
- Daily file system with automatic migration
- Dual persistence: Markdown for tasks, JSON for metadata
- Atomic file writes (temp + rename pattern)
- Soft boundaries on estimates (prompts, not enforcement)
- Human-editable file format (Markdown)
- Automatic report generation at day boundaries with mode breakdowns
- State history tracking for detailed analytics
- Independent timer tracking for tasks and subtasks
- Mode-based timer control (tasks only run in Working mode)

## Development

### Run tests

```bash
cargo test
```

### Run with debug logging

```bash
RUST_LOG=debug cargo run
```

### Format code

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

## Troubleshooting

### Emoji not displaying
Some terminals don't support emoji. Centre automatically falls back to ASCII characters (`*`, `+`, `!`).

### Editor doesn't open
Set your `$EDITOR` environment variable:
```bash
export EDITOR=vim
```

### Files corrupted
Centre creates `.bak` files when it detects parse errors. Check your centre directory (`~/.centre/` or `.centre/`) for backup files with timestamps.

### Wrong directory being used
Run `centre` to see which directory is active (shown at startup). Use `centre init` to create a local `.centre` directory for project-specific tasks.

## Roadmap

### v1.0 (Current)
- ✅ Core TUI with three panes (list, details, garden)
- ✅ Tasks and subtasks with independent timers
- ✅ Run/pause/done workflow
- ✅ Context mode switching (6 modes: Working, Lunch, Gym, Dinner, Personal, Sleep)
- ✅ Automatic task pause/resume on mode changes
- ✅ Mode time tracking with daily persistence
- ✅ Intelligent mode handling (tasks only start in Working mode)
- ✅ Estimate-hit modal
- ✅ External editor integration for notes
- ✅ Daily file system (YYYY-MM-DD.md)
- ✅ Automatic task migration between days
- ✅ Daily report generation with mode statistics (manual and automatic)
- ✅ Journal pane with cursor support
- ✅ Tags with visual badges
- ✅ State history tracking
- ✅ ETAs with time-of-day phases
- ✅ Archive system
- ✅ Local and global directory modes

### v1.1 (Planned)
- [ ] Config file support (`config.toml`)
- [ ] Weekly/monthly report aggregation
- [ ] Historical trend analysis
- [ ] Persistent collapse/expand state
- [ ] Enhanced add task forms with estimate input
- [ ] Improved navigation (PgUp/PgDn, vim-style)

### v1.2 (Future)
- [ ] Focus streaks visualization
- [ ] Tag-based filtering and views
- [ ] Blocker notes and dependencies
- [ ] Calendar view for historical data
- [ ] Export to CSV/JSON formats
- [ ] Pomodoro timer integration

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Credits

Built with:
- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [Chrono](https://github.com/chronotope/chrono) - Date and time

Inspired by calm, intentional productivity tools and the belief that estimates should guide, not constrain.

Centre recognizes that your day is more than just work—it's a rhythm of work, rest, nourishment, and recovery. Track it all.

---

**Track your whole day, not just your work. 🌱→🌿→🌵**
