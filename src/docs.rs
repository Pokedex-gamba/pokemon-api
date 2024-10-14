use utoipa::{
    openapi::{
        security::{Http, HttpAuthScheme, SecurityScheme},
        RefOr, Response,
    },
    Modify,
};

pub struct JwtGrantsAddon;

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

                    let responses = &mut operation.responses.responses;
                    responses.insert(
                        "400".into(),
                        RefOr::T(Response::new(
                            "Malformed authorization header or invalid jwt token",
                        )),
                    );
                    responses.insert(
                        "401".into(),
                        RefOr::T(Response::new("Missing authorization header")),
                    );
                    responses.insert(
                        "401".into(),
                        RefOr::T(Response::new("Insufficient permissions")),
                    );
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
