use anyhow::Result;

use super::{Cli, Commands};

/// Execute a CLI command and print JSON to stdout.
pub fn execute(command: &Commands, _args: &Cli) -> Result<()> {
    let response = match command {
        Commands::GetState => serde_json::json!({"ok": true, "data": "not yet implemented"}),
        Commands::GetSong => serde_json::json!({"ok": true, "data": "not yet implemented"}),
        Commands::ListScales => serde_json::json!({"ok": true, "data": "not yet implemented"}),
        Commands::ListInstruments => {
            serde_json::json!({"ok": true, "data": "not yet implemented"})
        }
        Commands::ListStrumPatterns => {
            serde_json::json!({"ok": true, "data": "not yet implemented"})
        }
        Commands::ListParts => serde_json::json!({"ok": true, "data": "not yet implemented"}),
        _ => serde_json::json!({"ok": true, "data": "not yet implemented"}),
    };

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
