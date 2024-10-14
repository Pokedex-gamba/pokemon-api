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

        for (_, path) in &mut openapi.paths.paths {
            for (_, operation) in &mut path.operations {
                let Some(securities) = &mut operation.security else {
                    continue;
                };

                let contains_self = securities
                    .iter()
                    .any(|security| security.value.contains_key("jwt_grants"));

                if contains_self {
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
        for (_, path) in &mut openapi.paths.paths {
            for (_, operation) in &mut path.operations {
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
