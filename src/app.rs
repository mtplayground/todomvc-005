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
    let toggle_all_action = ServerAction::<ToggleAll>::new();
    let update_action = ServerAction::<UpdateTodo>::new();
    let clear_completed_action = ServerAction::<ClearCompleted>::new();

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
    Effect::new(move |_| {
        let _ = toggle_all_action.version().get();
        set_refresh.update(|n| *n += 1);
    });
    Effect::new(move |_| {
        let _ = update_action.version().get();
        set_refresh.update(|n| *n += 1);
    });
    Effect::new(move |_| {
        let _ = clear_completed_action.version().get();
        set_refresh.update(|n| *n += 1);
    });

    let (filter, set_filter) = signal("all".to_string());

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
                    let current_filter = filter.get();
                    let filtered_list: Vec<Todo> = todo_list.iter().filter(|t| {
                        match current_filter.as_str() {
                            "active" => !t.completed,
                            "completed" => t.completed,
                            _ => true,
                        }
                    }).cloned().collect();
                    if todo_list.is_empty() {
                        view! {
                            <></>
                        }.into_any()
                    } else {
                        let footer_todos = todo_list.clone();
                        let all_completed = !todo_list.is_empty() && todo_list.iter().all(|t| t.completed);
                        let items_view: Vec<_> = filtered_list.iter().map(|todo| {
                            let todo = todo.clone();
                            let todo_id = todo.id;
                            let toggle_action2 = toggle_action;
                            let delete_action2 = delete_action;
                            let update_action2 = update_action;
                            view! {
                                <TodoItem
                                    todo=todo
                                    on_toggle=move || { toggle_action2.dispatch(ToggleTodo { id: todo_id }); }
                                    on_delete=move || { delete_action2.dispatch(DeleteTodo { id: todo_id }); }
                                    on_update=move |title| { update_action2.dispatch(UpdateTodo { id: todo_id, title }); }
                                />
                            }
                        }).collect();
                        view! {
                            <>
                                <section class="main">
                                    <input
                                        id="toggle-all"
                                        class="toggle-all"
                                        type="checkbox"
                                        prop:checked=all_completed
                                        on:change=move |_| {
                                            toggle_all_action.dispatch(ToggleAll { completed: !all_completed });
                                        }
                                    />
                                    <label for="toggle-all">"Mark all as complete"</label>
                                    <ul class="todo-list">
                                        {items_view}
                                    </ul>
                                </section>
                                <Footer todos=footer_todos filter=filter set_filter=set_filter clear_completed_action=clear_completed_action />
                            </>
                        }.into_any()
                    }
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn TodoItem(
    todo: Todo,
    on_toggle: impl Fn() + 'static + Send + Sync,
    on_delete: impl Fn() + 'static + Send + Sync,
    on_update: impl Fn(String) + 'static + Send + Sync,
) -> impl IntoView {
    let completed = todo.completed;
    let title = todo.title.clone();
    let todo_title_for_edit = todo.title.clone();
    let todo_title_for_escape = todo.title.clone();

    let (editing, set_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(todo.title.clone());

    use std::sync::Arc;
    let on_update = Arc::new(on_update);
    let on_update_blur = on_update.clone();
    let on_update_keydown = on_update.clone();

    let li_class = move || {
        match (completed, editing.get()) {
            (true, true) => "completed editing",
            (true, false) => "completed",
            (false, true) => "editing",
            (false, false) => "",
        }
    };

    view! {
        <li class=li_class>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=completed
                    on:change=move |_| on_toggle()
                />
                <label on:dblclick={
                    let todo_title_for_edit = todo_title_for_edit.clone();
                    move |_| {
                        set_edit_value.set(todo_title_for_edit.clone());
                        set_editing.set(true);
                    }
                }>{title.clone()}</label>
                <button class="destroy" on:click=move |_| on_delete()></button>
            </div>
            {move || {
                if editing.get() {
                    let on_update_kd = on_update_keydown.clone();
                    let on_update_bl = on_update_blur.clone();
                    let escape_title = todo_title_for_escape.clone();
                    view! {
                        <input
                            class="edit"
                            prop:value=edit_value
                            on:input=move |ev| set_edit_value.set(event_target_value(&ev))
                            on:keydown=move |ev: leptos::web_sys::KeyboardEvent| {
                                match ev.key().as_str() {
                                    "Enter" => {
                                        let val = edit_value.get();
                                        let val = val.trim().to_string();
                                        on_update_kd(val);
                                        set_editing.set(false);
                                    }
                                    "Escape" => {
                                        set_edit_value.set(escape_title.clone());
                                        set_editing.set(false);
                                    }
                                    _ => {}
                                }
                            }
                            on:blur=move |_| {
                                let val = edit_value.get();
                                let val = val.trim().to_string();
                                on_update_bl(val);
                                set_editing.set(false);
                            }
                        />
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }}
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

#[component]
fn Footer(
    todos: Vec<Todo>,
    filter: ReadSignal<String>,
    set_filter: WriteSignal<String>,
    clear_completed_action: ServerAction<ClearCompleted>,
) -> impl IntoView {
    let active_count = todos.iter().filter(|t| !t.completed).count();
    let has_completed = todos.iter().any(|t| t.completed);
    let item_text = if active_count == 1 { "item" } else { "items" };

    let filter_link = move |name: &'static str, label: &'static str| {
        let is_selected = move || filter.get() == name;
        let set_f = set_filter;
        view! {
            <li>
                <a
                    class=move || if is_selected() { "selected" } else { "" }
                    href={format!("#{}", name)}
                    on:click=move |_| set_f.set(name.to_string())
                >
                    {label}
                </a>
            </li>
        }
    };

    view! {
        <footer class="footer">
            <span class="todo-count">
                <strong>{active_count}</strong>
                {format!(" {} left", item_text)}
            </span>
            <ul class="filters">
                {filter_link("all", "All")}
                {filter_link("active", "Active")}
                {filter_link("completed", "Completed")}
            </ul>
            <Show when=move || has_completed>
                <button
                    class="clear-completed"
                    on:click=move |_| {
                        clear_completed_action.dispatch(ClearCompleted {});
                    }
                >
                    "Clear completed"
                </button>
            </Show>
        </footer>
    }
}
