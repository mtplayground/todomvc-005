use leptos::prelude::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<Todo>, ServerFnError> {
    use sqlx::Row;
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    let rows = sqlx::query("SELECT id, title, completed, display_order FROM todos ORDER BY display_order ASC, id ASC")
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

    let todos = rows.iter().map(|row| Todo {
        id: row.get("id"),
        title: row.get("title"),
        completed: row.get::<bool, _>("completed"),
        display_order: row.get("display_order"),
    }).collect();
    
    Ok(todos)
}

#[server(AddTodo, "/api")]
pub async fn add_todo(title: String) -> Result<Todo, ServerFnError> {
    use sqlx::Row;
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Title cannot be empty"));
    }
    
    let max_order_row = sqlx::query("SELECT MAX(display_order) as max_order FROM todos")
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    let max_order: Option<i64> = max_order_row.get("max_order");
    let next_order = max_order.unwrap_or(-1) + 1;
    
    let result = sqlx::query("INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)")
        .bind(&title)
        .bind(next_order)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    let id = result.last_insert_rowid();
    
    Ok(Todo {
        id,
        title,
        completed: false,
        display_order: next_order,
    })
}

#[server(UpdateTodo, "/api")]
pub async fn update_todo(id: i64, title: String) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    let title = title.trim().to_string();
    if title.is_empty() {
        return delete_todo(id).await;
    }
    
    sqlx::query("UPDATE todos SET title = ? WHERE id = ?")
        .bind(&title)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    Ok(())
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo(id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    Ok(())
}

#[server(ToggleTodo, "/api")]
pub async fn toggle_todo(id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    sqlx::query("UPDATE todos SET completed = NOT completed WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    Ok(())
}

#[server(ToggleAll, "/api")]
pub async fn toggle_all(completed: bool) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    sqlx::query("UPDATE todos SET completed = ?")
        .bind(completed)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    Ok(())
}

#[server(ClearCompleted, "/api")]
pub async fn clear_completed() -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("Database pool not found"))?;
    
    sqlx::query("DELETE FROM todos WHERE completed = TRUE")
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;
    
    Ok(())
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone()/>
                <MetaTags/>
                <link rel="stylesheet" href="/pkg/todomvc.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="TodoMVC"/>
        <section class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
            </header>
        </section>
    }
}
