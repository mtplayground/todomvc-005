use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub display_order: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl Filter {
    pub fn from_path(path: &str) -> Self {
        match path {
            "/active" => Filter::Active,
            "/completed" => Filter::Completed,
            _ => Filter::All,
        }
    }
}

// ============ Server Functions ============

#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<Todo>, ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    let rows = sqlx::query!(
        "SELECT id, title, completed, display_order FROM todos ORDER BY display_order ASC, id ASC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let todos = rows.into_iter().map(|r| Todo {
        id: r.id,
        title: r.title,
        completed: r.completed,
        display_order: r.display_order,
    }).collect();

    Ok(todos)
}

#[server(AddTodo, "/api")]
pub async fn add_todo(title: String) -> Result<Todo, ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("Title cannot be empty"));
    }

    let max_order: Option<i64> = sqlx::query_scalar!(
        "SELECT MAX(display_order) FROM todos"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let display_order = max_order.unwrap_or(0) + 1;

    let id = sqlx::query!(
        "INSERT INTO todos (title, completed, display_order) VALUES (?, FALSE, ?)",
        title,
        display_order
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .last_insert_rowid();

    Ok(Todo {
        id,
        title,
        completed: false,
        display_order,
    })
}

#[server(UpdateTodo, "/api")]
pub async fn update_todo(id: i64, title: String) -> Result<Option<Todo>, ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    let title = title.trim().to_string();
    if title.is_empty() {
        sqlx::query!("DELETE FROM todos WHERE id = ?", id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        return Ok(None);
    }

    sqlx::query!("UPDATE todos SET title = ? WHERE id = ?", title, id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = sqlx::query!(
        "SELECT id, title, completed, display_order FROM todos WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(row.map(|r| Todo {
        id: r.id,
        title: r.title,
        completed: r.completed,
        display_order: r.display_order,
    }))
}

#[server(DeleteTodo, "/api")]
pub async fn delete_todo(id: i64) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    sqlx::query!("DELETE FROM todos WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server(ToggleTodo, "/api")]
pub async fn toggle_todo(id: i64) -> Result<Todo, ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    sqlx::query!("UPDATE todos SET completed = NOT completed WHERE id = ?", id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = sqlx::query!(
        "SELECT id, title, completed, display_order FROM todos WHERE id = ?",
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Todo {
        id: row.id,
        title: row.title,
        completed: row.completed,
        display_order: row.display_order,
    })
}

#[server(ToggleAll, "/api")]
pub async fn toggle_all(completed: bool) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    sqlx::query!("UPDATE todos SET completed = ?", completed)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server(ClearCompleted, "/api")]
pub async fn clear_completed() -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;
    let pool = use_context::<SqlitePool>()
        .ok_or_else(|| ServerFnError::new("No database pool"))?;

    sqlx::query!("DELETE FROM todos WHERE completed = TRUE")
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

// ============ Components ============

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/todomvc.css"/>
        <Title text="TodoMVC - Leptos"/>
        <Router>
            <Routes>
                <Route path="/" view=TodoApp/>
                <Route path="/active" view=TodoApp/>
                <Route path="/completed" view=TodoApp/>
            </Routes>
        </Router>
    }
}

#[component]
fn TodoApp() -> impl IntoView {
    let location = use_location();
    let filter = create_memo(move |_| Filter::from_path(&location.pathname.get()));

    let todos = create_resource(
        || (),
        |_| async { get_todos().await.unwrap_or_default() },
    );

    let add_action = create_action(|title: &String| {
        let title = title.clone();
        async move { add_todo(title).await }
    });

    let delete_action = create_action(|id: &i64| {
        let id = *id;
        async move { delete_todo(id).await }
    });

    let toggle_action = create_action(|id: &i64| {
        let id = *id;
        async move { toggle_todo(id).await }
    });

    let update_action = create_action(|(id, title): &(i64, String)| {
        let id = *id;
        let title = title.clone();
        async move { update_todo(id, title).await }
    });

    let toggle_all_action = create_action(|completed: &bool| {
        let completed = *completed;
        async move { toggle_all(completed).await }
    });

    let clear_completed_action = create_action(|_: &()| async move {
        clear_completed().await
    });

    create_effect(move |_| {
        add_action.version().get();
        delete_action.version().get();
        toggle_action.version().get();
        update_action.version().get();
        toggle_all_action.version().get();
        clear_completed_action.version().get();
        todos.refetch();
    });

    let (new_todo, set_new_todo) = create_signal(String::new());

    let on_new_todo_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            let title = new_todo.get();
            let trimmed = title.trim().to_string();
            if !trimmed.is_empty() {
                add_action.dispatch(trimmed);
                set_new_todo.set(String::new());
            }
        }
    };

    let filtered_todos = create_memo(move |_| {
        let all = todos.get().unwrap_or_default();
        let f = filter.get();
        match f {
            Filter::All => all,
            Filter::Active => all.into_iter().filter(|t| !t.completed).collect(),
            Filter::Completed => all.into_iter().filter(|t| t.completed).collect(),
        }
    });

    let active_count = create_memo(move |_| {
        todos.get().unwrap_or_default().iter().filter(|t| !t.completed).count()
    });

    let completed_count = create_memo(move |_| {
        todos.get().unwrap_or_default().iter().filter(|t| t.completed).count()
    });

    let all_completed = create_memo(move |_| {
        let all = todos.get().unwrap_or_default();
        !all.is_empty() && all.iter().all(|t| t.completed)
    });

    let has_todos = create_memo(move |_| {
        !todos.get().unwrap_or_default().is_empty()
    });

    view! {
        <section class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
                <input
                    class="new-todo"
                    placeholder="What needs to be done?"
                    autofocus
                    prop:value=new_todo
                    on:input=move |ev| set_new_todo.set(event_target_value(&ev))
                    on:keydown=on_new_todo_keydown
                />
            </header>

            <Show when=move || has_todos.get()>
                <section class="main">
                    <input
                        id="toggle-all"
                        class="toggle-all"
                        type="checkbox"
                        prop:checked=all_completed
                        on:change=move |_| {
                            let all_done = all_completed.get();
                            toggle_all_action.dispatch(!all_done);
                        }
                    />
                    <label for="toggle-all">"Mark all as complete"</label>

                    <ul class="todo-list">
                        <For
                            each=move || filtered_todos.get()
                            key=|todo| todo.id
                            children=move |todo| {
                                view! {
                                    <TodoItem
                                        todo=todo
                                        on_toggle=move |id| toggle_action.dispatch(id)
                                        on_delete=move |id| delete_action.dispatch(id)
                                        on_update=move |(id, title)| update_action.dispatch((id, title))
                                    />
                                }
                            }
                        />
                    </ul>
                </section>

                <footer class="footer">
                    <span class="todo-count">
                        <strong>{active_count}</strong>
                        {move || if active_count.get() == 1 { " item left" } else { " items left" }}
                    </span>

                    <ul class="filters">
                        <li>
                            <a href="/" class=move || if filter.get() == Filter::All { "selected" } else { "" }>
                                "All"
                            </a>
                        </li>
                        <li>
                            <a href="/active" class=move || if filter.get() == Filter::Active { "selected" } else { "" }>
                                "Active"
                            </a>
                        </li>
                        <li>
                            <a href="/completed" class=move || if filter.get() == Filter::Completed { "selected" } else { "" }>
                                "Completed"
                            </a>
                        </li>
                    </ul>

                    <Show when=move || completed_count.get() != 0>
                        <button
                            class="clear-completed"
                            on:click=move |_| { clear_completed_action.dispatch(()); }
                        >
                            "Clear completed"
                        </button>
                    </Show>
                </footer>
            </Show>
        </section>

        <footer class="info">
            <p>"Double-click to edit a todo"</p>
            <p>"Built with "<a href="https://leptos.dev">"Leptos"</a></p>
        </footer>
    }
}

#[component]
fn TodoItem(
    todo: Todo,
    on_toggle: impl Fn(i64) + 'static + Clone,
    on_delete: impl Fn(i64) + 'static + Clone,
    on_update: impl Fn((i64, String)) + 'static + Clone,
) -> impl IntoView {
    let (editing, set_editing) = create_signal(false);
    let (edit_text, set_edit_text) = create_signal(todo.title.clone());

    let todo_id = todo.id;
    let todo_completed = todo.completed;
    let todo_title_clone = todo.title.clone();
    let todo_title_display = todo.title.clone();

    let on_toggle_for_cb = on_toggle.clone();
    let on_delete_for_cb = on_delete.clone();
    let on_update_for_commit = on_update.clone();

    let on_toggle_cb = move |_: ev::Event| on_toggle_for_cb(todo_id);
    let on_delete_cb = move |_: ev::MouseEvent| on_delete_for_cb(todo_id);

    let start_edit = {
        let title = todo_title_clone.clone();
        move |_: ev::MouseEvent| {
            set_edit_text.set(title.clone());
            set_editing.set(true);
        }
    };

    let do_commit = move || {
        let text = edit_text.get();
        let trimmed = text.trim().to_string();
        on_update_for_commit((todo_id, trimmed));
        set_editing.set(false);
    };

    let do_commit_for_keydown = do_commit.clone();
    let escape_title = todo.title.clone();

    let on_edit_keydown = move |ev: ev::KeyboardEvent| {
        match ev.key().as_str() {
            "Enter" => do_commit_for_keydown(),
            "Escape" => {
                set_edit_text.set(escape_title.clone());
                set_editing.set(false);
            }
            _ => {}
        }
    };

    let on_edit_blur = move |_: ev::FocusEvent| {
        if editing.get() {
            do_commit();
        }
    };

    let li_class = move || {
        let mut classes = Vec::new();
        if todo_completed {
            classes.push("completed");
        }
        if editing.get() {
            classes.push("editing");
        }
        classes.join(" ")
    };

    view! {
        <li class=li_class>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=todo_completed
                    on:change=on_toggle_cb
                />
                <label on:dblclick=start_edit>
                    {todo_title_display}
                </label>
                <button
                    class="destroy"
                    on:click=on_delete_cb
                />
            </div>
            <input
                class="edit"
                prop:value=edit_text
                on:input=move |ev| set_edit_text.set(event_target_value(&ev))
                on:keydown=on_edit_keydown
                on:blur=on_edit_blur
            />
        </li>
    }
}
