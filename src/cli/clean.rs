use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing::info;

/// Execute the clean command
pub async fn execute(pool: &SqlitePool) -> Result<()> {
    info!("Cleaning up processed files...");

    // Delete executed actions older than 7 days
    let result = sqlx::query(
        r#"
        DELETE FROM pending_actions
        WHERE executed = 1 AND created_at < datetime('now', '-7 days')
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to clean up old actions")?;

    let deleted_actions = result.rows_affected();

    // Also clean up completed file records older than 30 days
    let file_result = sqlx::query(
        r#"
        DELETE FROM file_registry
        WHERE status = 'completed' AND created_at < datetime('now', '-30 days')
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to clean up old file records")?;

    let deleted_files = file_result.rows_affected();

    println!("\n╔════════════════════════════════════════════╗");
    println!("║     Cleanup Complete                       ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("  Deleted old actions:     {}", deleted_actions);
    println!("  Deleted old records:     {}", deleted_files);
    println!();

    Ok(())
}
