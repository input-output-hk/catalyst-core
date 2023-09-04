//! Custom middleware to catch any unimplemented endpoints and then pass them to the
//! legacy axum implementation.
//!
//! Allows us to have 1 API and seamlessly migrate unconverted requests.
use poem::{async_trait, error::NotFoundError, Endpoint, Middleware, Request, Result};

/// Middleware to chain call Axum if endpoint is not found.
pub struct ChainAxum;

impl ChainAxum {
    /// Create new `ChainAxum` middleware with any value.
    pub fn new() -> Self {
        ChainAxum {}
    }
}

impl<E> Middleware<E> for ChainAxum
where
    E: Endpoint,
{
    type Output = ChainAxumEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        ChainAxumEndpoint { inner: ep }
    }
}

/// Endpoint for ChainAxum middleware.
pub struct ChainAxumEndpoint<E> {
    inner: E,
}

#[async_trait]
impl<E> Endpoint for ChainAxumEndpoint<E>
where
    E: Endpoint,
{
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        // Need to copy all the request parts because its consumed by the `call`.
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, body);

        match self.inner.call(req).await {
            Ok(res) => Ok(res),
            Err(err) => {
                // Only if the error is a 404 (Not found) then try and chain to axum, handler.
                //if err.is::<NotFoundError>() {
                // TODO Call Axum endpoints here.
                //    Err(err)
                //} else {
                Err(err)
                // }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[tokio::test]
    async fn test_axum_chain() {
        //#[handler(internal)]
        //async fn index(req: &Request) {
        //    assert_eq!(req.extensions().get::<i32>(), Some(&100));
        //}

        //let cli = TestClient::new(index.with(ChainAxum::new(100i32)));
        //cli.get("/").send().await.assert_status_is_ok();
    }
}
