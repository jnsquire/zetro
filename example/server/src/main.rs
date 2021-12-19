mod generated;
mod routes;

use generated::{generate_routes, ZetroContext};
use routes::{Db, Mutations, Queries};
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() {
    let mut ctx = ZetroContext::new();

    let db = Mutex::new(Db::new());

    ctx.insert(db);

    let queries = Queries {};
    let mutations = Mutations {};

    let routes = warp::get()
        .and(warp::path::end())
        .and(warp::fs::dir("../client/dist"))
        .or(warp::get()
            .and(warp::path("build"))
            .and(warp::fs::dir("../client/dist/build")))
        .or(warp::post()
            .and(warp::path("api"))
            .and(warp::path::end())
            .and(generate_routes(ctx, queries, mutations))); // The magic happens here

    println!("Visit http://127.0.0.1:8090");
    warp::serve(routes).run(([127, 0, 0, 1], 8090)).await;
}
