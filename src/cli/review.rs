use anyhow::{Context, Result};
use sqlx::SqlitePool;
use tokio::fs;
use tracing::info;

use crate::storage::PendingAction;

/// Execute the review command
pub async fn execute(pool: &SqlitePool, apply: bool) -> Result<()> {
    if apply {
        info!("Executing pending actions...");
        execute_actions(pool).await?;
    } else {
        info!("Reviewing pending actions (dry-run)...");
        show_pending_actions(pool).await?;
    }

    Ok(())
}

/// Show pending actions without executing
async fn show_pending_actions(pool: &SqlitePool) -> Result<()> {
    let actions = sqlx::query_as::<_, PendingAction>(
        r#"
        SELECT * FROM pending_actions
        WHERE executed = 0
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch pending actions")?;

    if actions.is_empty() {
        println!("No pending actions.");
        return Ok(());
    }

    println!("\nPending Actions:");
    println!("================\n");

    for action in actions {
        println!(
            "[{}] {} -> {}",
            action.action_type, action.source_path, action.destination_path
        );
    }

    println!("\nRun 'dtriage review --apply' to execute these actions.\n");

    Ok(())
}

/// Execute pending actions
async fn execute_actions(pool: &SqlitePool) -> Result<()> {
    let actions = sqlx::query_as::<_, PendingAction>(
        r#"
        SELECT * FROM pending_actions
        WHERE executed = 0
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch pending actions")?;

    if actions.is_empty() {
        println!("No pending actions to execute.");
        return Ok(());
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for action in actions {
        info!("Executing: {} -> {}", action.source_path, action.destination_path);

        // Ensure destination directory exists
        if let Some(parent) = std::path::Path::new(&action.destination_path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                println!("  ✗ Failed to create directory: {}", e);
                error_count += 1;
                continue;
            }
        }

        // Perform atomic file move
        match fs::rename(&action.source_path, &action.destination_path).await {
            Ok(_) => {
                info!("Successfully moved file");
                println!("  ✓ Moved: {} -> {}", action.source_path, action.destination_path);
                success_count += 1;

                // Mark action as executed
                sqlx::query(
                    r#"
                    UPDATE pending_actions
                    SET executed = 1
                    WHERE id = ?
                    "#,
                )
                .bind(action.id)
                .execute(pool)
                .await
                .context("Failed to update action status")?;

                // Update file registry status
                sqlx::query(
                    r#"
                    UPDATE file_registry
                    SET status = 'completed', updated_at = CURRENT_TIMESTAMP
                    WHERE id = ?
                    "#,
                )
                .bind(action.file_id)
                .execute(pool)
                .await
                .ok();
            }
            Err(e) => {
                tracing::error!("Failed to move file: {}", e);
                println!("  ✗ Failed: {} -> {} ({})", action.source_path, action.destination_path, e);
                error_count += 1;
            }
        }
    }

    println!(
        "\nCompleted: {} succeeded, {} failed",
        success_count, error_count
    );

    Ok(())
}
