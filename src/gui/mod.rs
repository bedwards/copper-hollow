pub mod app;
pub mod header;
pub mod pattern_view;
pub mod theme;
pub mod tracks;
pub mod widgets;

use anyhow::Result;

/// Launch the GUI application.
pub fn run() -> Result<()> {
    tracing::info!("GUI mode — not yet implemented");
    Ok(())
}
