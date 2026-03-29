//! Integration tests for TodoMVC server functions
//! These tests require a running SQLite database connection.

#[cfg(test)]
#[cfg(feature = "ssr")]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                display_order INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create table");

        pool
    }

    #[tokio::test]
    async fn test_create_and_read_todo() {
        let pool = setup_db().await;

        // Insert a todo
        let todo = sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, 1) RETURNING id, title, completed, display_order",
            "Test todo"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert todo");

        assert_eq!(todo.title, "Test todo");
        assert!(!todo.completed);

        // Read all todos
        let todos = sqlx::query!("SELECT id, title, completed, display_order FROM todos")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch todos");

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Test todo");
    }

    #[tokio::test]
    async fn test_toggle_todo() {
        let pool = setup_db().await;

        let todo = sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, 1) RETURNING id, title, completed, display_order",
            "Toggle me"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert todo");

        let id = todo.id;

        // Toggle
        let updated = sqlx::query!(
            "UPDATE todos SET completed = NOT completed WHERE id = ? RETURNING id, title, completed, display_order",
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to toggle todo");

        assert!(updated.completed);
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let pool = setup_db().await;

        let todo = sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, 1) RETURNING id, title, completed, display_order",
            "Delete me"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert todo");

        let id = todo.id;

        sqlx::query!("DELETE FROM todos WHERE id = ?", id)
            .execute(&pool)
            .await
            .expect("Failed to delete todo");

        let todos = sqlx::query!("SELECT id FROM todos WHERE id = ?", id)
            .fetch_all(&pool)
            .await
            .expect("Failed to query");

        assert!(todos.is_empty());
    }

    #[tokio::test]
    async fn test_clear_completed() {
        let pool = setup_db().await;

        sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES ('Active', FALSE, 1), ('Done', TRUE, 2)"
        )
        .execute(&pool)
        .await
        .expect("Failed to insert todos");

        sqlx::query!("DELETE FROM todos WHERE completed = TRUE")
            .execute(&pool)
            .await
            .expect("Failed to clear completed");

        let todos = sqlx::query!("SELECT id, title FROM todos")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch");

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Active");
    }

    #[tokio::test]
    async fn test_toggle_all() {
        let pool = setup_db().await;

        sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES ('A', FALSE, 1), ('B', FALSE, 2)"
        )
        .execute(&pool)
        .await
        .expect("Failed to insert todos");

        sqlx::query!("UPDATE todos SET completed = TRUE")
            .execute(&pool)
            .await
            .expect("Failed to toggle all");

        let todos = sqlx::query!("SELECT completed FROM todos")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch");

        assert!(todos.iter().all(|t| t.completed));
    }

    #[tokio::test]
    async fn test_update_todo_title() {
        let pool = setup_db().await;

        let todo = sqlx::query!(
            "INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, 1) RETURNING id, title, completed, display_order",
            "Old title"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert todo");

        let id = todo.id;

        let updated = sqlx::query!(
            "UPDATE todos SET title = ? WHERE id = ? RETURNING id, title, completed, display_order",
            "New title",
            id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to update todo");

        assert_eq!(updated.title, "New title");
    }
}
