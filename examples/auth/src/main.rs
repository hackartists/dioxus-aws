#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing;

#[derive(Clone, Routable, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    tracing::info!("starting app");
    #[cfg(feature = "web")]
    dioxus_aws::launch(App);

    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let sdk_config = aws_config::load_from_env().await;
                let config = aws_sdk_dynamodb::config::Builder::from(&sdk_config).build();

                let session_layer = tower_sessions::SessionManagerLayer::new(
                    tower_sessions_dynamodb_store::DynamoDBStore::new(
                        aws_sdk_dynamodb::Client::from_conf(config),
                        tower_sessions_dynamodb_store::DynamoDBStoreProps {
                            table_name: option_env!("SESSION_TABLE")
                                .unwrap_or("session-dev")
                                .to_string(),
                            partition_key: tower_sessions_dynamodb_store::DynamoDBStoreKey {
                                name: "id".to_string(),
                                prefix: Some("SESSIONS::TOWER::".to_string()),
                                suffix: None,
                            },
                            sort_key: None,
                            expirey_name: "expire_at".to_string(),
                            data_name: "session_data".to_string(),
                            create_key_max_retry_attempts: 5,
                        },
                    ),
                );

                dioxus_aws::AppBuilder::new(App)
                    .layer(session_layer)
                    .serve()
            });
    }
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let mut text = use_signal(|| String::from("..."));

    rsx! {
        Link {
            to: Route::Blog {
                id: count()
            },
            "Go to blog"
        }
        div {
            h1 { "High-Five counter: {count}" }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
            button {
                onclick: move |_| async move {
                    if let Ok(data) = get_server_data().await {
                        tracing::info!("Client received: {}", data);
                        text.set(data.clone());
                        post_server_data(data).await.unwrap();
                    }
                },
                "Get Server Data"
            }
            p { "Server data: {text}"}
        }
    }
}

#[server(PostServerData)]
async fn post_server_data(data: String) -> Result<(), ServerFnError> {
    tracing::info!("Server received: {}", data);
    Ok(())
}

#[server(GetServerData)]
async fn get_server_data() -> Result<String, ServerFnError> {
    Ok("Hello from the server!".to_string())
}
