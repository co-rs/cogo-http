#![doc(html_root_url = "https://docs.rs/hyper/v0.10.16")]
//#![cfg_attr(test, deny(missing_docs))]
//#![cfg_attr(test, deny(warnings))]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! # Hyper
//!
//! mco-http is a fast, modern HTTP implementation written in and for Rust. It
//! is a low-level typesafe abstraction over raw HTTP, providing an elegant
//! layer over "stringly-typed" HTTP.
//!
//! mco-http offers both a [Client](client/index.html) and a
//! [Server](server/index.html) which can be used to drive complex web
//! applications written entirely in Rust.
//!
//! ## Internal Design
//!
//! mco-http is designed as a relatively low-level wrapper over raw HTTP. It should
//! allow the implementation of higher-level abstractions with as little pain as
//! possible, and should not irrevocably hide any information from its users.
//!
//! ### Common Functionality
//!
//! Functionality and code shared between the Server and Client implementations
//! can be found in `src` directly - this includes `NetworkStream`s, `Method`s,
//! `StatusCode`, and so on.
//!
//! #### Methods
//!
//! Methods are represented as a single `enum` to remain as simple as possible.
//! Extension Methods are represented as raw `String`s. A method's safety and
//! idempotence can be accessed using the `safe` and `idempotent` methods.
//!
//! #### StatusCode
//!
//! Status codes are also represented as a single, exhaustive, `enum`. This
//! representation is efficient, typesafe, and ergonomic as it allows the use of
//! `match` to disambiguate known status codes.
//!
//! #### Headers
//!
//! Hyper's [header](header/index.html) representation is likely the most
//! complex API exposed by Hyper.
//!
//! Hyper's headers are an abstraction over an internal `HashMap` and provides a
//! typesafe API for interacting with headers that does not rely on the use of
//! "string-typing."
//!
//! Each HTTP header in mco-http has an associated type and implementation of the
//! `Header` trait, which defines an HTTP headers name as a string, how to parse
//! that header, and how to format that header.
//!
//! Headers are then parsed from the string representation lazily when the typed
//! representation of a header is requested and formatted back into their string
//! representation when headers are written back to the client.
//!
//! #### NetworkStream and NetworkAcceptor
//!
//! These are found in `src/net.rs` and define the interface that acceptors and
//! streams must fulfill for them to be used within Hyper. They are by and large
//! internal tools and you should only need to mess around with them if you want to
//! mock or replace `TcpStream` and `TcpAcceptor`.
//!
//! ### Server
//!
//! Server-specific functionality, such as `Request` and `Response`
//! representations, are found in in `src/server`.
//!
//! #### Handler + Server
//!
//! A `Handler` in mco-http accepts a `Request` and `Response`. This is where
//! user-code can handle each connection. The server accepts connections in a
//! task pool with a customizable number of threads, and passes the Request /
//! Response to the handler.
//!
//! #### Request
//!
//! An incoming HTTP Request is represented as a struct containing
//! a `Reader` over a `NetworkStream`, which represents the body, headers, a remote
//! address, an HTTP version, and a `Method` - relatively standard stuff.
//!
//! `Request` implements `Reader` itself, meaning that you can ergonomically get
//! the body out of a `Request` using standard `Reader` methods and helpers.
//!
//! #### Response
//!
//! An outgoing HTTP Response is also represented as a struct containing a `Writer`
//! over a `NetworkStream` which represents the Response body in addition to
//! standard items such as the `StatusCode` and HTTP version. `Response`'s `Writer`
//! implementation provides a streaming interface for sending data over to the
//! client.
//!
//! One of the traditional problems with representing outgoing HTTP Responses is
//! tracking the write-status of the Response - have we written the status-line,
//! the headers, the body, etc.? mco-http tracks this information statically using the
//! type system and prevents you, using the type system, from writing headers after
//! you have started writing to the body or vice versa.
//!
//! mco-http does this through a phantom type parameter in the definition of Response,
//! which tracks whether you are allowed to write to the headers or the body. This
//! phantom type can have two values `Fresh` or `Streaming`, with `Fresh`
//! indicating that you can write the headers and `Streaming` indicating that you
//! may write to the body, but not the headers.
//!
//! ### Client
//!
//! Client-specific functionality, such as `Request` and `Response`
//! representations, are found in `src/client`.
//!
//! #### Request
//!
//! An outgoing HTTP Request is represented as a struct containing a `Writer` over
//! a `NetworkStream` which represents the Request body in addition to the standard
//! information such as headers and the request method.
//!
//! Outgoing Requests track their write-status in almost exactly the same way as
//! outgoing HTTP Responses do on the Server, so we will defer to the explanation
//! in the documentation for server Response.
//!
//! Requests expose an efficient streaming interface instead of a builder pattern,
//! but they also provide the needed interface for creating a builder pattern over
//! the API exposed by core Hyper.
//!
//! #### Response
//!
//! Incoming HTTP Responses are represented as a struct containing a `Reader` over
//! a `NetworkStream` and contain headers, a status, and an http version. They
//! implement `Reader` and can be read to get the data out of a `Response`.
//!


#[macro_use]
pub extern crate http;

extern crate base64;
extern crate time;
#[macro_use]
extern crate url;
extern crate unicase;
extern crate httparse;
extern crate traitobject;
extern crate typeable;

#[cfg_attr(test, macro_use)]
extern crate language_tags;
#[macro_use]
extern crate mime as mime_crate;

#[macro_use]
extern crate log;

#[cfg(all(test, feature = "nightly"))]
extern crate test;


use std::hash::Hasher;
use std::io::Write;
pub use url::Url;
pub use client::Client;
pub use error::{Result, Error};
pub use method::Method::{Get, Head, Post, Delete};
pub use status::StatusCode::{Ok, BadRequest, NotFound};
pub use server::Server;
pub use language_tags::LanguageTag;

macro_rules! todo (
    ($($arg:tt)*) => (if cfg!(not(ndebug)) {
        trace!("TODO: {:?}", format_args!($($arg)*))
    })
);

//#[cfg(test)]
#[macro_use]
pub mod mock;
#[doc(hidden)]
pub mod buffer;
pub mod client;
pub mod error;
pub mod method;
pub mod proto;
pub mod net;
pub mod server;
pub mod status;
pub mod uri;
pub mod version;
pub mod multipart;
pub mod query;
pub mod path;
pub mod route;
pub mod runtime;
pub mod json;


/// Re-exporting the mime crate, for convenience.
pub mod mime {
    pub use mime_crate::*;
}


fn _assert_types() {
    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    _assert_send::<Client>();
    _assert_send::<client::Request<net::Fresh>>();
    _assert_send::<client::Response>();
    _assert_send::<error::Error>();
    _assert_send::<crate::client::pool::Pool<crate::net::DefaultConnector>>();

    _assert_sync::<Client>();
    _assert_sync::<error::Error>();
    _assert_sync::<crate::client::pool::Pool<crate::net::DefaultConnector>>();
}


#[macro_export]
macro_rules! header_value {
    ($v:expr) => { HeaderValue::from_str($v).unwrap() };
}

#[macro_export]
macro_rules! header_name {
    ($v:expr) => { http::header::HeaderName::from_str($v)};
}
