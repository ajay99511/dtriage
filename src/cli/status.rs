use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tracing::info;

/// Execute the status command
pub async fn execute(pool: &SqlitePool) -> Result<()> {
    info!("Fetching triage status...");

    // Get pending actions count
    let pending_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pending_actions WHERE executed = 0",
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch pending count")?;

    // Get total files count
    let total_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM file_registry")
        .fetch_one(pool)
        .await
        .context("Failed to fetch total files")?;

    // Get categorized files count
    let categorized_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM file_registry WHERE status = 'categorized'",
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch categorized count")?;

    // Get completed files count
    let completed_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM file_registry WHERE status = 'completed'",
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch completed count")?;

    // Get recent pending actions
    let recent_actions = sqlx::query_as::<_, crate::storage::PendingAction>(
        r#"
        SELECT * FROM pending_actions
        WHERE executed = 0
        ORDER BY created_at DESC
        LIMIT 5
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    println!("\n╔════════════════════════════════════════════╗");
    println!("║     Downloads Triage Status                ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("  Files Tracked:      {}", total_files.0);
    println!("  Categorized:        {}", categorized_count.0);
    println!("  Completed:          {}", completed_count.0);
    println!("  Pending Actions:    {}", pending_count.0);
    println!();

    if !recent_actions.is_empty() {
        println!("  Recent Pending Actions:");
        println!("  ─────────────────────────────────────────");
        for action in recent_actions {
            println!(
                "    [{}] {}",
                action.action_type, action.destination_path
            );
        }
        println!();
    }

    println!("  Run 'dtriage review' to see all pending actions");
    println!("  Run 'dtriage review --apply' to execute them");
    println!();

    Ok(())
}
