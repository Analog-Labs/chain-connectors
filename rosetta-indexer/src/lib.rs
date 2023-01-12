use anyhow::Result;
use tide::{Body, Request, Response};

mod db;

pub use crate::db::Indexer;

pub async fn server(indexer: Indexer) -> Result<tide::Server<Indexer>> {
    let mut app = tide::with_state(indexer);
    app.at("/search/transactions").post(search);
    Ok(app)
}

async fn search(mut request: Request<Indexer>) -> tide::Result {
    let req = request.body_json().await?;
    let res = request.state().search(&req).await;
    Ok(match res {
        Ok(res) => Response::builder(200)
            .body(Body::from_json(&res).unwrap())
            .build(),
        Err(err) => {
            let error = rosetta_types::Error {
                code: 500,
                message: format!("{}", err),
                description: None,
                retriable: false,
                details: None,
            };
            Response::builder(500)
                .body(Body::from_json(&error).unwrap())
                .build()
        }
    })
}
