#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::SqlitePoolOptions;
    use todomvc::app::App;

    let conf = get_configuration(None).await.expect("Failed to get configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:todos.db".to_string());

    let pool = SqlitePoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || provide_context(pool.clone())
            },
            App,
        )
        .fallback(leptos_axum::render_app_to_stream(
            leptos_options.clone(),
            App,
        ))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind address");
    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.expect("Server failed");
}

#[cfg(not(feature = "ssr"))]
fn main() {}
