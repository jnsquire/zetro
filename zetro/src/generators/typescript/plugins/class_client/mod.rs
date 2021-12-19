use convert_case::{Case, Casing};

use crate::common::schema::{
    fields::FieldKind,
    routes::{RouteKind, ZetroRoute},
};

/// Generates frontend typescript code to query a server using the Zetro spec.
pub(crate) fn class_client(
    scope: &mut Vec<String>,
    queries: &Vec<ZetroRoute>,
    mutations: &Vec<ZetroRoute>,
    untagged_repr: bool,
    should_mangle: bool,
) {
    scope.push(generate_api_client_interface(should_mangle).to_string());

    scope.push(String::from("/* ============ Queries ============ */"));
    scope.push(generate_client_class(
        "ZetroQuery",
        RouteKind::Query.to_method_code(),
        &queries,
        untagged_repr,
        should_mangle,
    ));
    scope.push(String::from("/* ============ End Queries ============ */"));

    scope.push(String::from("/* ============ Mutations ============ */"));
    scope.push(generate_client_class(
        "ZetroMutation",
        RouteKind::Mutation.to_method_code(),
        &mutations,
        untagged_repr,
        should_mangle,
    ));
    scope.push(String::from(
        "/* ============ End Mutations ============ */",
    ));
}

pub(super) fn generate_api_client_interface(should_mangle: bool) -> String {
    format!(
        "/** Users must implement this interface to use `ZetroQuery` and `ZetroMutation` */
export interface IZetroClient {{
\t/**
\t * Body is array-encoded data for the request. The return value MUST be the
\t * response JSON body of the server. Implementors can add authentication
\t * information to the request. This interface can be easily mocked for tests.
\t * Note that a non-200 response status (even in the case of a malformed request)
\t * MUST be considered an unexpected error and be handled accordingly.
\t * In other words, only forward the parsed JSON body to this method if the
\t * status is 200 OK.
\t */
\tmakeRequest{}: (body: any) => Promise<any>;
}}",
        if should_mangle { "_" } else { "" }
    )
}

/// Generates an ES6 class that contains all the API routes.
pub(super) fn generate_client_class(
    name: &str,
    method_code: u8,
    routes: &Vec<ZetroRoute>,
    untagged_repr: bool,
    should_mangle: bool,
) -> String {
    // Contains class methods for individual routes.
    let mut methods: Vec<String> = Vec::new();

    for route in routes {
        let route_name_min = format!(
            "{}{}",
            route.name.to_case(Case::Camel),
            if should_mangle { "_" } else { "" }
        );
        let route_encrypted = route.encrypt_route_name();

        // Expression for request and response body
        // It's not as complicated as it looks. The expressions
        // change based on 3 factors:
        // 1. If the thing is a struct or nested object, call the serialize
        //    function for that thing. Add a `?` depending on its nullability.
        // 2. If the thing is something else, don't call the function in (1).
        //    that is, use the thing as-is.
        // 3. If not untagged_repr, don't call the function in (1).
        let request_body_expr = if untagged_repr {
            match &route.request_body.kind {
                FieldKind::StructValue(struct_name) => {
                    if route.request_body.is_multiple {
                        format!(
                        "requestBody{}.map(function (elem: any) {{ return serialize{}(elem); }})",
                        if route.request_body.is_nullable {
                            "?"
                        } else {
                            ""
                        },
                        struct_name,
                    )
                    } else {
                        format!("serialize{}(requestBody)", struct_name)
                    }
                }
                FieldKind::NestedObject(s) => {
                    if route.request_body.is_multiple {
                        format!(
                        "requestBody{}.map(function (elem: any) {{ return serialize{}(elem); }})",
                        if route.request_body.is_nullable {
                            "?"
                        } else {
                            ""
                        },
                        s.name
                    )
                    } else {
                        format!("serialize{}(requestBody)", s.name)
                    }
                }
                _ => String::from("requestBody"),
            }
        } else {
            String::from("requestBody")
        };
        let response_body_expr = if untagged_repr {
            match &route.response_body.kind {
                FieldKind::StructValue(struct_name) => {
                    if route.request_body.is_multiple {
                        format!(
                            "item[1]{}.map(function (elem: any) {{ return deserialize{}(elem); }})",
                            if route.response_body.is_nullable {
                                "?"
                            } else {
                                ""
                            },
                            struct_name
                        )
                    } else {
                        format!("deserialize{}(item[1])", struct_name)
                    }
                }
                FieldKind::NestedObject(s) => {
                    if route.request_body.is_multiple {
                        format!(
                            "item[1]{}.map(function (elem: any) {{ return deserialize{}(elem); }})",
                            if route.response_body.is_nullable {
                                "?"
                            } else {
                                ""
                            },
                            s.name
                        )
                    } else {
                        format!("deserialize{}(item[1])", s.name)
                    }
                }
                _ => String::from("item[1]"),
            }
        } else {
            String::from("item[1]")
        };

        // Method code generation :O
        methods.push(format!(
            "\t{0}(requestBody{1}: {2}): {3}<T & {{{0}: {4}}}> {{
\t\tthis.state_.push([\"{5}\", {6}]);
\t\tthis.parsers_.push(function (resultObj: any, item: any) {{
\t\t\tresultObj.{0} = {7};
\t\t}})
\t\treturn this as any;
\t}}",
            // Route name. This also becomes the name of the class method
            route_name_min,
            // Request nullability
            if route.request_body.is_nullable {
                "?"
            } else {
                ""
            },
            // Request type
            route.request_body.to_ts_dtype(),
            // Return type: this class' name
            name,
            // Return type: response type of this route
            route.response_body.to_ts_dtype(),
            route_encrypted,
            request_body_expr,
            response_body_expr,
        ));
    }

    let class_code = format!(
        "export class {0}<T = unknown> {{
\tprivate state_: any[] = [];
\tprivate parsers_: ((returnObject: any, item: any) => void)[] = [];
\tprivate readonly client_: IZetroClient;

\tconstructor(client: IZetroClient) {{
\t\tthis.client_ = client;
\t}}

{1}

/*
 * Excecutes the request and returns the response.
 * If the call was unsuccessful, an error of type ZetroServerError
 * will be thrown.
 */
\tasync fetch{3}(): Promise<T> {{
\t\ttry {{
\t\t\tconst result = await this.client_.makeRequest{3}([{2}, this.state_]);
\t\t\tif (result[1] != null) {{
\t\t\t\t// Error
\t\t\t\tthrow {{code{3}: result[1][0], message{3}: result[1][1]}}
\t\t\t}}
\t\t\tconst data = result[0];
\t\t\tconst returnObject = {{}};
\t\t\tfor (let i = 0; i < data.length; i++) {{
\t\t\t\tthis.parsers_[i](returnObject, data[i]);
\t\t\t}}
\t\t\treturn returnObject as any;
\t\t}} catch (e) {{
\t\t\tthrow {{code{3}: e.code{3} || -1, message{3}: e.message{3} || \"An unexpected error occurred.\"}};
\t\t}}
}}
}}",
        name,
        methods.join("\n"),
        method_code,
        if should_mangle { "_" } else { "" }
    );

    class_code
}
