use paste::paste;

macro_rules! build_resp_inner {
    ($name:ident ,$path:ident) => {
        #[macro_export]
        macro_rules! $name {
            () => {
                actix_web::HttpResponse::$path().finish()
            };
            ($message:expr) => {
                actix_web::HttpResponse::$path().body($message)
            };
        }
    };
}

macro_rules! build_resp {
    ($name:ident, $path:ident) => {
        paste! {
            build_resp_inner!([<$name _macro>], $path);
            #[allow(unused_imports)]
            pub use [<$name _macro>] as $name;
            #[macro_export]
            macro_rules! [<$name _json_macro>] {
                () => {
                    actix_web::HttpResponse::$path().insert_header((actix_web::http::header::ContentType::json())).body("{}")
                };
                ($message:expr) => {
                    actix_web::HttpResponse::$path().json($message)
                };
            }
            #[allow(unused_imports)]
            pub use [<$name _json_macro>] as [<$name _json>];
        }
    };
}

build_resp!(resp_500_InternalServerError, InternalServerError);
build_resp!(resp_400_BadRequest, BadRequest);
build_resp!(resp_200_Ok, Ok);
build_resp!(resp_401_Unauthorized, Unauthorized);
build_resp!(resp_403_Forbidden, Forbidden);
build_resp!(resp_404_NotFound, NotFound);

#[macro_export]
macro_rules! yeet_error_macro {
    ($result:expr) => {
        match $result {
            Ok(data) => data,
            Err(e) => return e,
        }
    };
}
#[allow(unused_imports)]
pub use yeet_error_macro as yeet_error;
