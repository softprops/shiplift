//! Representations of various client errors

use hyper::status::StatusCode;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Io(::std::io::Error);
        Hyper(::hyper::Error);
        RustcSerializeDecoder(::rustc_serialize::json::DecoderError);
        RustcSerializeEncoder(::rustc_serialize::json::EncoderError);
        RustcSerializeParser(::rustc_serialize::json::ParserError);
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
    }

}

