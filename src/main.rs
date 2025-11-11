mod app;
mod domain;
mod input;
mod notifications;
mod persistence;
mod report;
mod ticker;
mod ui;

use app::AppState;
use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use persistence::{ensure_centre_dir, get_centre_dir, init_local_centre, journal_file, load_and_migrate};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[derive(Parser)]
#[command(name = "centre")]
#[command(about = "A calm, terminal-based daily focus manager with time tracking", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a local .centre directory in the current directory
    Init,
    /// Generate a daily report with statistics
    Report {
        /// Date to generate report for (YYYY-MM-DD format). Defaults to today.
        #[arg(short, long)]
        date: Option<String>,
        /// Output file path. Defaults to ~/.centre/report-YYYY-MM-DD.md
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            // Initialize local .centre directory
            let centre_dir = init_local_centre()?;
            println!("Initialized centre directory: {}", centre_dir.display());
            println!();
            println!("Centre will now use this local directory for task storage.");
            println!("Run 'centre' to start tracking tasks.");
            Ok(())
        }
        Some(Commands::Report { date, output }) => {
            // Generate daily report
            let report_date = if let Some(date_str) = date {
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| anyhow::anyhow!("Invalid date format. Use YYYY-MM-DD: {}", e))?
            } else {
                chrono::Local::now().date_naive()
            };

            let output_path = output.map(std::path::PathBuf::from);

            println!("Generating report for {}...", report_date);
            let report_path = report::generate_report(Some(report_date), output_path)?;
            println!("Report generated: {}", report_path.display());
            Ok(())
        }
        None => {
            // Run the normal TUI application
            run_tui()
        }
    }
}

fn run_tui() -> Result<()> {
    // Ensure centre directory exists
    ensure_centre_dir()?;

    // Show which directory we're using
    let centre_dir = get_centre_dir()?;
    eprintln!("Using centre directory: {}", centre_dir.display());

    // Load and migrate tasks (uses new daily file format)
    let (tasks, done_today, archived_today) = load_and_migrate()?;

    // Load journal
    let journal_content = match std::fs::read_to_string(journal_file()?) {
        Ok(content) => content,
        Err(_) => String::new(), // Empty journal if file doesn't exist
    };

    // Create app state
    let mut app = AppState::new(tasks, done_today, archived_today, journal_content);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Move all tasks to idle on exit (to track state properly)
    app.auto_idle_all();

    // Save on exit
    if let Err(e) = app.save() {
        eprintln!("Error saving state: {}", e);
    }
    if let Err(e) = app.save_journal() {
        eprintln!("Error saving journal: {}", e);
    }

    // Print any errors
    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut AppState) -> Result<()> {
    let tick_rate = ticker::tick_duration();

    loop {
        // Check for midnight crossing - force restart
        if app.has_day_changed() {
            // Generate report for the day that just passed
            let yesterday = app.file_date; // The date we were tracking
            if let Err(e) = report::generate_report(Some(yesterday), None) {
                eprintln!("Warning: Failed to generate report for {}: {}", yesterday, e);
            } else {
                eprintln!("Generated report for {}", yesterday);
            }

            // Show modal forcing user to restart
            app.ui_mode = domain::UiMode::DayChanged;
        }

        // Render
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events with timeout for ticking
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Only process key press events (ignore key release)
                if key.kind == KeyEventKind::Press {
                    // If day changed, only allow quit
                    if app.ui_mode == domain::UiMode::DayChanged {
                        if key.code == event::KeyCode::Char('q') || key.code == event::KeyCode::Esc {
                            return Ok(());
                        }
                        continue; // Ignore all other keys
                    }

                    // Handle notes editing specially - need to disable raw mode
                    if app.ui_mode == domain::UiMode::Normal
                        && (key.code == event::KeyCode::Char('n')
                            || key.code == event::KeyCode::Char('N'))
                    {
                        // Disable raw mode and leave alternate screen
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                        // Handle the key (which will spawn editor)
                        let should_quit = input::handle_key(app, key)?;

                        // Re-enable raw mode and alternate screen
                        enable_raw_mode()?;
                        execute!(io::stdout(), EnterAlternateScreen)?;

                        // Clear and redraw
                        terminal.clear()?;

                        if should_quit {
                            return Ok(());
                        }
                    } else {
                        // Normal key handling
                        let should_quit = input::handle_key(app, key)?;
                        if should_quit {
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Tick timers
        app.tick();

        // Check for estimate hits
        app.check_estimate_hits();

        // Check for idle time
        app.check_idle_time();

        // Autosave if needed
        if app.needs_save {
            app.save()?;
        }
        if app.journal_needs_save {
            app.save_journal()?;
        }
    }
}
