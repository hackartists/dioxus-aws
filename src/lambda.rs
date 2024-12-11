#[allow(missing_docs)]
pub struct LambdaAdapter<'a, R, S> {
    service: S,
    _phantom_data: std::marker::PhantomData<&'a R>,
}

impl<'a, R, S, E> From<S> for LambdaAdapter<'a, R, S>
where
    S: tower::Service<lambda_http::Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: lambda_http::IntoResponse,
{
    fn from(service: S) -> Self {
        LambdaAdapter {
            service,
            _phantom_data: std::marker::PhantomData,
        }
    }
}

impl<'a, R, S, E> tower::Service<lambda_http::LambdaEvent<lambda_http::request::LambdaRequest>>
    for LambdaAdapter<'a, R, S>
where
    S: tower::Service<lambda_http::Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: lambda_http::IntoResponse,
{
    type Response = lambda_http::aws_lambda_events::apigw::ApiGatewayProxyResponse;

    type Error = E;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'a>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(
        &mut self,
        req: lambda_http::LambdaEvent<lambda_http::request::LambdaRequest>,
    ) -> Self::Future {
        // tracing::debug!("Incoming request: {:?}", req);
        let stage = match req.payload {
            lambda_http::request::LambdaRequest::ApiGatewayV1(ref req) => {
                req.request_context.stage.clone()
            }
            _ => None,
        };

        let mut event: lambda_http::Request = req.payload.into();
        let method = event.method().to_string();
        // tracing::debug!("Parsed incoming event: {:?}", event);
        let path_and_query = event.uri().path_and_query().expect("path_and_query");
        *event.uri_mut() = http::Uri::from(path_and_query.clone());

        if let Some(stage) = stage {
            let uri = event.uri().to_string().clone();
            let stage = format!("/{}", stage);
            *event.uri_mut() = (&uri).replacen(&stage, "", 1).parse().unwrap();
        }

        let uri = event.uri().to_string().clone();
        // tracing::debug!("manipulated event requests: {:?}", event);

        let call = self
            .service
            .call(lambda_http::RequestExt::with_lambda_context(
                event,
                req.context,
            ));

        Box::pin(async move {
            let res = call.await?;
            let res = res.into_response().await;
            let status_code = res.status().as_u16() as i64;
            let mut headers = res.headers().clone();
            let body = Some(res.clone().into_body());

            let is_base64_encoded = headers
                .get("content-type")
                .map(|v| {
                    v.to_str()
                        .map(|v| v.contains("image") || v.contains("octet-stream"))
                        .unwrap_or(false)
                })
                .unwrap_or_default();

            if is_base64_encoded {
                tracing::debug!("Binary response: {} {}", method, uri);
                headers.insert(
                    "content-encoding",
                    http::header::HeaderValue::from_static("base64"),
                );
            }

            let res = lambda_http::aws_lambda_events::apigw::ApiGatewayProxyResponse {
                is_base64_encoded,
                status_code,
                headers,
                body,
                multi_value_headers: Default::default(),
            };
            if uri.contains("/ko") {
                tracing::debug!(
                    "Outgoing response: {} {} \n data:{:?}",
                    method,
                    uri,
                    match &res.body {
                        Some(lambda_http::Body::Text(body)) => body.to_string(),
                        _ => "".to_string(),
                    },
                );
            }

            Ok(res)
        })
    }
}
