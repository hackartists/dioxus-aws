use dioxus::prelude::*;

#[cfg(feature = "lambda")]
mod lambda;

#[doc = include_str!("../docs/launch.md")]
pub fn launch(app: fn() -> Element) {
    #[cfg(feature = "web")]
    dioxus::launch(app);

    #[cfg(all(not(feature = "lambda"), feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "lambda")]
    {
        use axum::routing::*;
        use dioxus_fullstack::prelude::*;

        struct TryIntoResult(Result<ServeConfig, dioxus_fullstack::UnableToLoadIndex>);

        impl TryInto<ServeConfig> for TryIntoResult {
            type Error = dioxus_fullstack::UnableToLoadIndex;

            fn try_into(self) -> Result<ServeConfig, Self::Error> {
                self.0
            }
        }

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                use self::lambda::LambdaAdapter;
                let app = Router::new().serve_dioxus_application(
                    TryIntoResult(ServeConfigBuilder::default().build()),
                    app,
                );

                tracing::info!("Running in lambda mode");
                lambda_runtime::run(LambdaAdapter::from(app)).await.unwrap();
            });
    };
}
