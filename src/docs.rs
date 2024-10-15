use utoipa::{
    openapi::{
        security::{Http, HttpAuthScheme, SecurityScheme},
        RefOr, Response,
    },
    Modify,
};

pub struct JwtGrantsAddon;

fn append_response(
    responses: &mut utoipa::openapi::Responses,
    key: impl Into<String>,
    response: &str,
) {
    responses
        .responses
        .entry(key.into())
        .and_modify(|ref_or_response| match ref_or_response {
            RefOr::Ref(_) => unimplemented!("$ref in response is not supported by JwtGrantsAddon"),
            RefOr::T(oa_response) => {
                oa_response.description.push_str("<br>or<br>");
                oa_response.description.push_str(response);
            }
        })
        .or_insert(RefOr::T(Response::new(response)));
}

impl Modify for JwtGrantsAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt_grants",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );

        for path in openapi.paths.paths.values_mut() {
            for operation in path.operations.values_mut() {
                let Some(securities) = &operation.security else {
                    continue;
                };

                let mut contains_self = false;
                let mut required_grants = securities
                    .iter()
                    .filter(|security| security.value.contains_key("jwt_grants"))
                    .fold("<pre>Required grants:".to_string(), |mut acc, security| {
                        contains_self = true;

                        for grants in security
                            .value
                            .iter()
                            .filter(|(k, _)| k.as_str() == "jwt_grants")
                            .map(|(_, v)| v)
                        {
                            acc.push_str("\n\t");
                            if grants.is_empty() {
                                acc.push_str("None");
                            } else {
                                acc.push_str(&grants.join(", "));
                            }
                        }

                        acc
                    });
                required_grants.push_str("</pre>");

                if contains_self {
                    if let Some(description) = operation.description.as_mut() {
                        if !description.is_empty() {
                            description.push_str("<br><br>");
                        }

                        description.push_str(&required_grants);
                    } else {
                        operation.description = Some(required_grants);
                    }

                    let responses = &mut operation.responses;
                    append_response(
                        responses,
                        "400",
                        "Malformed authorization header or invalid jwt token",
                    );
                    append_response(responses, "401", "Missing authorization header");
                    append_response(responses, "401", "Insufficient permissions");
                }
            }
        }
    }
}

pub struct AutoTagAddon;

impl Modify for AutoTagAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        for path in openapi.paths.paths.values_mut() {
            for operation in path.operations.values_mut() {
                let tags = operation.tags.take().unwrap_or_default();

                let mut new_tags = tags
                    .into_iter()
                    .filter(|t| !t.starts_with("crate::"))
                    .collect::<Vec<_>>();
                new_tags.push("All routes".into());

                operation.tags = Some(new_tags);
            }
        }
    }
}
