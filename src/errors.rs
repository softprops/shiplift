//! Representations of various client errors

use hyper::status::StatusCode;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        EnvVar(::std::env::VarError);
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        HyperParser(::hyper::error::ParseError);
        OpenSSL(::openssl::error::ErrorStack) #[cfg(feature = "ssl")];
        SerdeJsonError(::serde_json::error::Error);
    }

    errors {
        HyperFault(code: StatusCode) {
            description("HyperFault")
                display("{}", code)
        }

        Utf8 {
            description("Error while trying to handle non-utf8 string")
                display("Error while trying to handle non-utf8 string")
        }

        JsonFieldMissing(name: &'static str) {
            description("JSON Field missing")
                display("JSON Field '{}' missing", name)
        }

        JsonTypeError(fieldname: &'static str, expectedtype: &'static str) {
            description("JSON Field has wrong type")
                display("JSON Field '{}' has wrong type, expected: {}", fieldname, expectedtype)
        }

        NoHostString {
            description("Failed to find a host string")
                display("Failed to find a host string")
        }

        NoPort {
            description("Failed to find a port")
                display("Failed to find a port")
        }
    }

}
