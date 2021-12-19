use convert_case::{Case, Casing};

use crate::{
    common::schema::routes::{RouteKind, ZetroRoute},
    utilities::{parse_bool, PluginCall},
};

/// Generates backend rust code to run an HTTP API.
pub(crate) fn warp(
    plug: &PluginCall,
    scope: &mut Vec<String>,
    queries: &Vec<ZetroRoute>,
    mutations: &Vec<ZetroRoute>,
) {
    // Whether to use fnv::FnvHashMap instead of std::collections::HashMap
    // in the Context struct
    let use_fnv = match plug.args.get("fnv") {
        Some(v) => parse_bool(&v.to_lowercase()).expect("`fnv` must be 'true' or 'false'"),
        None => false,
    };

    // First add the context struct in scope...
    let (ctx_struct, ctx_impl) = generate_context_struct(use_fnv);
    scope.push(ctx_struct);
    scope.push(ctx_impl);

    // ...and the reply generator function
    let (data_reply_fn, error_reply_fn) = generate_reply_fns();
    scope.push(data_reply_fn);
    scope.push(error_reply_fn);

    // ...then generate traits for queries and mutations
    scope.push(generate_routes_trait("ZetroQueries", &queries));
    scope.push(generate_routes_trait("ZetroMutations", &mutations));

    // ...and finally generate the routing function
    scope.push(generate_routing_fn(&queries, &mutations));
}

/// Generates the `ZetroContext` struct and impl block that is passed into every
/// route. The first return value is the struct and the second is the impl block.
fn generate_context_struct(use_fnv: bool) -> (String, String) {
    let ctx_struct = format!(
        "pub struct ZetroContext {{
\tdata: {}<std::any::TypeId, Box<dyn std::any::Any + Sync + Send>>,
}}",
        if use_fnv {
            "fnv::FnvHashMap"
        } else {
            "std::collections::HashMap"
        }
    );

    let ctx_impl = format!(
        "impl ZetroContext {{
\tpub fn new() -> Self {{
\t\tZetroContext {{
\t\t\tdata: {},
\t\t}}
\t}}

\tpub fn insert<T>(&mut self, item: T)
\twhere T: std::any::Any + Sync + Send,
\t{{
\t\tself.data.insert(std::any::TypeId::of::<T>(), Box::new(item));
\t}}
    
\tpub fn get<T>(&self) -> &T
\twhere T: std::any::Any + Sync + Send,
\t{{
\t\tself.data
\t\t\t.get(&std::any::TypeId::of::<T>())
\t\t\t.unwrap()
\t\t\t.downcast_ref::<T>()
\t\t\t.unwrap()
\t\t}}
}}",
        if use_fnv {
            "fnv::FnvHashMap::default()"
        } else {
            "std::collections::HashMap::new()"
        }
    );

    (ctx_struct, ctx_impl)
}

/// Generates private utility functions to build spec-compliant responses
/// The first function is _generate_data_reply and the second is
/// _generate_error_reply
fn generate_reply_fns() -> (String, String) {
    // Data reply function
    let data_reply_fn = String::from(
        "fn _generate_data_reply(data: Vec<serde_json::Value>) -> warp::reply::Response {
\tlet serialized = serde_json::to_string(&(
\t\t&data,
\t\t&serde_json::Value::Null,
\t))
\t.unwrap();

\twarp::http::Response::builder()
\t\t.status(200)
\t\t.body(warp::hyper::body::Body::from(serialized))
\t\t.unwrap()
}",
    );

    // Error reply function
    let error_reply_fn = String::from(
        "fn _generate_error_reply(code: i16, message: &str) -> warp::reply::Response {
\tlet serialized = serde_json::to_string(&(
\t\t&serde_json::Value::Null,
\t\t&(&code, message),
\t\t))
\t\t.unwrap();

\twarp::http::Response::builder()
\t\t.status(200)
\t\t.body(warp::hyper::body::Body::from(serialized))
\t\t.unwrap()
}",
    );

    (data_reply_fn, error_reply_fn)
}

/// Generates a trait of routes which can be implemented to serve API requests.
fn generate_routes_trait(trait_name: &str, routes: &Vec<ZetroRoute>) -> String {
    let mut trait_fns: Vec<String> = Vec::new();

    for route in routes {
        trait_fns.push(format!(
            "\tasync fn {}<'a>(ctx: &'a ZetroContext, request: {}) -> Result<{}, ZetroServerError>;",
            route.name.to_case(Case::Snake),
            route.request_body.to_rust_dtype(),
            route.response_body.to_rust_dtype(),
        ));
    }

    format!(
        "{}pub trait {} {{\n{}\n}}",
        "#[async_trait::async_trait]\n",
        trait_name,
        trait_fns.join("\n\n")
    )
}

/// The routing function is where the meat of the work happens.
/// That function is responsible for receiving a ZetroContext, implementations
/// of `ZetroQueries` and `ZetroMutations`, and returning a warp route.
fn generate_routing_fn(queries: &Vec<ZetroRoute>, mutations: &Vec<ZetroRoute>) -> String {
    let mut query_match_arms: Vec<String> = Vec::new();
    let mut mutation_match_arms: Vec<String> = Vec::new();

    for query in queries {
        let route_encrypted = query.encrypt_route_name();

        // Add match arms to match route ID with route
        query_match_arms.push(format!(
            "
                        // '{}' route:
                        \"{}\" => {{
                            let route_body = serde_json::from_value::<{}>(route_body);
                            if route_body.is_err() {{
                                return _generate_error_reply(400, \"Bad request\");
                            }}
                            let route_body = route_body.unwrap();
                            let result = Q::{}(&ctx, route_body).await;
                            match result {{
                                Err(e) => return _generate_error_reply(e.code, &e.message),
                                Ok(d) => serde_json::to_value((route_name, &d)).unwrap()
                            }}
                        }}",
            query.name.clone(),
            route_encrypted,
            query.request_body.to_rust_dtype(),
            query.name.to_case(Case::Snake), // Only the function for the route will be renamed.
        ));
    }
    for mutation in mutations {
        let route_encrypted = mutation.encrypt_route_name();

        // Add match arms to match route ID with route
        mutation_match_arms.push(format!(
            "
                        // '{}' route:
                        \"{}\" => {{
                            let route_body = serde_json::from_value::<{}>(route_body);
                            if route_body.is_err() {{
                                return _generate_error_reply(400, \"Bad request\");
                            }}
                            let route_body = route_body.unwrap();
                            let result = M::{}(&ctx, route_body).await;
                            match result {{
                                Err(e) => return _generate_error_reply(e.code, &e.message),
                                Ok(d) => serde_json::to_value((route_name, &d)).unwrap()
                            }}
                        }}",
            mutation.name.clone(),
            route_encrypted,
            mutation.request_body.to_rust_dtype(),
            mutation.name.to_case(Case::Snake), // Only the function for the route will be renamed.
        ));
    }

    // I could convert these to tabs, but is it really worth the effort?
    let routing_fn = format!(
        "pub fn generate_routes<Q, M>(ctx: ZetroContext, _queries: Q, _mutations: M) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
where Q: ZetroQueries, M: ZetroMutations,
{{
        use std::sync::Arc;

        let ctx = Arc::new(ctx);

        warp::any()
            .and(warp::any().map(move || Arc::clone(&ctx)))
            .and(warp::body::bytes())
            .then(|ctx: Arc<ZetroContext>, body: bytes::Bytes| async move {{
                let mut retval: Vec<serde_json::Value> = Vec::new();
                let request_payload = serde_json::from_slice::<(u8, Vec<serde_json::Value>)>(&body);
                if request_payload.is_err() {{
                    return _generate_error_reply(400, \"Bad request\");
                }}
                // Determines whether the request is a query or mutation
                let (method_code, operations) = request_payload.unwrap();
                for op in operations {{
                    if !op.is_array() {{
                        return _generate_error_reply(400, \"Operations must be an array\");
                    }}
                    let arr = op.as_array().unwrap();
                    let route_name = arr.get(0);
                    let route_body = arr.get(1);

                    if route_name.is_none() || route_body.is_none() {{
                        return _generate_error_reply(400, \"Route name and route body are mandatory\");
                    }}

                    let route_name = route_name.unwrap();
                    let route_body = route_body.unwrap().to_owned();

                    if !route_name.is_string() {{
                        return _generate_error_reply(400, \"Route name must be string\");
                    }}
                    let route_name = route_name.as_str().unwrap();
                    if method_code == {} {{
                        // Handle query
                        retval.push(match route_name {{
                            {}
    
                            _ => {{
                                return _generate_error_reply(400, \"Unrecognized route name\");
                            }}
                        }});
                    }} else if method_code == {} {{
                        // Handle mutation
                        retval.push(match route_name {{
                            {}
    
                            _ => {{
                                return _generate_error_reply(400, \"Unrecognized route name\");
                            }}
                        }});
                    }} else {{
                        return _generate_error_reply(400, \"Bad request\");
                    }}
                }}
                _generate_data_reply(retval)
            }})
        }}",
        RouteKind::Query.to_method_code(),
        query_match_arms.join("\n"),
        RouteKind::Mutation.to_method_code(),
        mutation_match_arms.join("\n"),
    );

    routing_fn
}
