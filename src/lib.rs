use dioxus::prelude::*;

#[cfg(feature = "lambda")]
mod lambda;

#[doc = include_str!("../docs/launch.md")]
pub fn launch(app: fn() -> Element) {
    #[cfg(not(feature = "lambda"))]
    dioxus::launch(app);

    #[cfg(feature = "lambda")]
    {
        use axum::routing::*;
        use dioxus_fullstack::prelude::*;

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                use self::lambda::LambdaAdapter;
                let app = Router::new()
                    .serve_dioxus_application(ServeConfigBuilder::default().build(), app);

                tracing::info!("Running in lambda mode");
                lambda_runtime::run(LambdaAdapter::from(app)).await.unwrap();
            });
    };
}
