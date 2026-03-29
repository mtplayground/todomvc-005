// Integration tests for TodoMVC SQLite operations
// These tests use an in-memory SQLite database

#[cfg(test)]
mod tests {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sqlx::Row;
    use std::str::FromStr;

    async fn setup_test_db() -> sqlx::SqlitePool {
        let connect_options = SqliteConnectOptions::from_str("sqlite::memory:")
            .expect("Failed to parse database URL")
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await
            .expect("Failed to connect to database");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                display_order INTEGER NOT NULL DEFAULT 0
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create todos table");

        pool
    }

    #[tokio::test]
    async fn test_add_todo() {
        let pool = setup_test_db().await;

        sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
            .bind("Test todo")
            .bind(0i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        let row = sqlx::query("SELECT COUNT(*) as count FROM todos")
            .fetch_one(&pool)
            .await
            .expect("Failed to count todos");

        let count: i64 = row.get("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_toggle_todo() {
        let pool = setup_test_db().await;

        let result = sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
            .bind("Toggle me")
            .bind(0i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        let id = result.last_insert_rowid();

        sqlx::query("UPDATE todos SET completed = NOT completed WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Failed to toggle todo");

        let row = sqlx::query("SELECT completed FROM todos WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch todo");

        let completed: bool = row.get("completed");
        assert!(completed);
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let pool = setup_test_db().await;

        let result = sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
            .bind("Delete me")
            .bind(0i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        let id = result.last_insert_rowid();

        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Failed to delete todo");

        let row = sqlx::query("SELECT COUNT(*) as count FROM todos")
            .fetch_one(&pool)
            .await
            .expect("Failed to count todos");

        let count: i64 = row.get("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_update_todo_title() {
        let pool = setup_test_db().await;

        let result = sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
            .bind("Old title")
            .bind(0i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        let id = result.last_insert_rowid();

        sqlx::query("UPDATE todos SET title = ? WHERE id = ?")
            .bind("New title")
            .bind(id)
            .execute(&pool)
            .await
            .expect("Failed to update todo");

        let row = sqlx::query("SELECT title FROM todos WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch todo");

        let title: String = row.get("title");
        assert_eq!(title, "New title");
    }

    #[tokio::test]
    async fn test_clear_completed() {
        let pool = setup_test_db().await;

        // Add 2 completed and 1 active todo
        sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, TRUE, ?)")
            .bind("Completed 1")
            .bind(0i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, TRUE, ?)")
            .bind("Completed 2")
            .bind(1i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
            .bind("Active")
            .bind(2i64)
            .execute(&pool)
            .await
            .expect("Failed to insert todo");

        sqlx::query("DELETE FROM todos WHERE completed = TRUE")
            .execute(&pool)
            .await
            .expect("Failed to clear completed");

        let row = sqlx::query("SELECT COUNT(*) as count FROM todos")
            .fetch_one(&pool)
            .await
            .expect("Failed to count todos");

        let count: i64 = row.get("count");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_toggle_all() {
        let pool = setup_test_db().await;

        for i in 0..3 {
            sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
                .bind(format!("Todo {}", i))
                .bind(i as i64)
                .execute(&pool)
                .await
                .expect("Failed to insert todo");
        }

        sqlx::query("UPDATE todos SET completed = ?")
            .bind(true)
            .execute(&pool)
            .await
            .expect("Failed to toggle all");

        let row = sqlx::query("SELECT COUNT(*) as count FROM todos WHERE completed = TRUE")
            .fetch_one(&pool)
            .await
            .expect("Failed to count completed todos");

        let count: i64 = row.get("count");
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_get_todos_order() {
        let pool = setup_test_db().await;

        for (i, title) in ["C", "A", "B"].iter().enumerate() {
            sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
                .bind(*title)
                .bind(i as i64)
                .execute(&pool)
                .await
                .expect("Failed to insert todo");
        }

        let rows = sqlx::query("SELECT title FROM todos ORDER BY display_order ASC, id ASC")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch todos");

        let titles: Vec<String> = rows.iter().map(|r| r.get("title")).collect();
        assert_eq!(titles, vec!["C", "A", "B"]);
    }
}
