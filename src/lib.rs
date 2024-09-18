use dioxus::prelude::*;

#[cfg(feature = "lambda")]
mod lambda;

#[doc = include_str!("../docs/launch.md")]
pub fn launch(app: fn() -> Element) {
    #[cfg(not(feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    {
        use axum::routing::*;
        use dioxus_fullstack::prelude::*;

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let virtual_dom_factory = move || VirtualDom::new(app);

                use std::path::PathBuf;
                let conf = ServeConfigBuilder::default().assets_path(PathBuf::from(
                    std::env::var("ASSETS_PATH").unwrap_or(
                        #[cfg(feature = "lambda")]
                        "./".to_string(),
                        #[cfg(not(feature = "lambda"))]
                        "./dist".to_string(),
                    ),
                ));

                let app = Router::new()
                    .serve_dioxus_application(conf.build(), virtual_dom_factory)
                    .await;

                #[cfg(feature = "session")]
                let app = app.layer(
                    tower_sessions::SessionManagerLayer::new(
                        tower_sessions_dynamodb_store::DynamoDBStore::new(
                            tower_sessions_dynamodb_store::DynamoDBClient::new()
                                .await
                                .get_client(),
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
                    )
                    .with_expiry(tower_sessions::Expiry::OnInactivity(
                        time::Duration::hours(6),
                    )),
                );

                #[cfg(feature = "lambda")]
                {
                    use self::lambda::LambdaAdapter;

                    tracing::info!("Running in lambda mode");
                    lambda_runtime::run(LambdaAdapter::from(app)).await.unwrap();
                }

                #[cfg(not(feature = "lambda"))]
                {
                    tracing::info!("Running in axum mode");
                    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
                    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

                    axum::serve(listener, app.into_make_service())
                        .await
                        .unwrap();
                }
            });
    };
}
