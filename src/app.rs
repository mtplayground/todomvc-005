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

    let (refresh, set_refresh) = signal(0u32);

    let add_action = ServerAction::<AddTodo>::new();
    let toggle_action = ServerAction::<ToggleTodo>::new();
    let delete_action = ServerAction::<DeleteTodo>::new();

    // Refresh when any action completes
    Effect::new(move |_| {
        let _ = add_action.version().get();
        set_refresh.update(|n| *n += 1);
    });
    Effect::new(move |_| {
        let _ = toggle_action.version().get();
        set_refresh.update(|n| *n += 1);
    });
    Effect::new(move |_| {
        let _ = delete_action.version().get();
        set_refresh.update(|n| *n += 1);
    });

    let todos = Resource::new(
        move || refresh.get(),
        |_| async move { get_todos().await.unwrap_or_default() },
    );

    view! {
        <Title text="TodoMVC"/>
        <section class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
                <TodoInput add_action=add_action />
            </header>
            <Suspense fallback=|| view! { <></> }>
                {move || {
                    let todo_list = todos.get().unwrap_or_default();
                    let has_todos = !todo_list.is_empty();
                    view! {
                        <Show when=move || has_todos>
                            <section class="main">
                                <ul class="todo-list">
                                    {todo_list.iter().map(|todo| {
                                        let todo = todo.clone();
                                        let todo_id = todo.id;
                                        view! {
                                            <TodoItem
                                                todo=todo
                                                on_toggle=move || { toggle_action.dispatch(ToggleTodo { id: todo_id }); }
                                                on_delete=move || { delete_action.dispatch(DeleteTodo { id: todo_id }); }
                                                on_update=move |_title| {}
                                            />
                                        }
                                    }).collect_view()}
                                </ul>
                            </section>
                        </Show>
                    }
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn TodoItem(
    todo: Todo,
    on_toggle: impl Fn() + 'static,
    on_delete: impl Fn() + 'static,
    on_update: impl Fn(String) + 'static,
) -> impl IntoView {
    let completed = todo.completed;
    let title = todo.title.clone();

    view! {
        <li class={if completed { "completed" } else { "" }}>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=completed
                    on:change=move |_| on_toggle()
                />
                <label on:dblclick=move |_| { let _ = &on_update; }>{title.clone()}</label>
                <button class="destroy" on:click=move |_| on_delete()></button>
            </div>
        </li>
    }
}

#[component]
fn TodoInput(add_action: ServerAction<AddTodo>) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());

    view! {
        <input
            class="new-todo"
            placeholder="What needs to be done?"
            prop:value=input_value
            on:input=move |ev| set_input_value.set(event_target_value(&ev))
            on:keydown=move |ev| {
                if ev.key() == "Enter" {
                    let val = input_value.get();
                    let val = val.trim().to_string();
                    if !val.is_empty() {
                        add_action.dispatch(AddTodo { title: val });
                        set_input_value.set(String::new());
                    }
                }
            }
            autofocus
        />
    }
}
