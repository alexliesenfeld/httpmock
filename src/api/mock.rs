use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{Method, Regex};
use crate::server::data::{
    MockDefinition, MockMatcherFunction, MockServerHttpResponse, Pattern, RequestRequirements,
};
use crate::util::Join;
use crate::MockServer;
use std::time::Duration;

/// The [Mock](struct.Mock.html) structure holds a definition for a request/response scenario
/// that can be used to configure a [MockServer](struct.MockServer.html).
///
/// This structure provides methods starting with `expect` in their name to
/// define requirements for HTTP requests that the server will respond to. On the other hand,
/// methods starting with `return` in their name define what data the mock server will put into
/// the corresponding HTTP response.
///
/// Because of this naming scheme, this structure is said to privide access to the
/// "expect/return" API of `httpmock`.
///
/// # Example
/// ```
/// // Arrange
/// use httpmock::{MockServer, Mock};
///
/// let mock_server = MockServer::start();
///
/// let mock = Mock::new()
///     .expect_path_contains("/search")
///     .expect_query_param("query", "metallica")
///     .return_status(202)
///     .create_on(&mock_server);
///
/// // Act: Send the HTTP request
/// let response = isahc::get(mock_server.url("/search?query=metallica")).unwrap();
///
/// // Assert
/// assert_eq!(response.status(), 202);
/// assert_eq!(mock.times_called(), 1);
/// ```
/// Observe how [Mock::create_on](struct.Mock.html#method.create_on) is used to create a mock object
/// on the server. After the call completes, the mock server will start serving HTTP requests
/// as specified in the [Mock](struct.Mock.html) instance.
///
/// The [Mock::create_on](struct.Mock.html#method.create_on) method also returns a mock reference
/// that identifies the mock object on the server. It can be used to fetch related information
/// from the server, such as the number of times the mock was served
/// (see [MockRef::times_called](struct.MockRef.html#method.times_called)). You can also use it
/// to explicitly delete the mock object from the server
/// (see [MockRef::delete](struct.MockRef.html#method.delete)).
pub struct Mock {
    mock: MockDefinition,
}

/// Represents a reference to the mock object on a [MockServer](struct.MockServer.html).
/// It can be used to spy on the mock and also perform some management operations, such as
/// deleting the mock from the [MockServer](struct.MockServer.html).
///
/// # Example
/// ```
/// // Arrange
/// use httpmock::{MockServer, Mock};
///
/// let mock_server = MockServer::start();
///
/// let mut mock = mock_server.mock(|when, then|{
///    when.path("/test");
///    then.status(202);
/// });
///
/// // Send a first request, then delete the mock from the mock and send another request.
/// let response1 = isahc::get(mock_server.url("/test")).unwrap();
///
/// // Fetch how often this mock has been called from the server until now
/// assert_eq!(mock.times_called(), 1);
///
/// // Delete the mock from the mock server
/// mock.delete();
///
/// let response2 = isahc::get(mock_server.url("/test")).unwrap();
///
/// // Assert
/// assert_eq!(response1.status(), 202);
/// assert_eq!(response2.status(), 404);
/// ```
pub struct MockRef<'a> {
    id: usize,
    mock_server: &'a MockServer,
}

impl<'a> MockRef<'a> {
    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mut mock = mock_server.mock(|when, then| {
    ///     when.path("/times_called");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// isahc::get(mock_server.url("/times_called")).unwrap();
    ///
    /// // Assert: Fetch how often this mock has been called from the server until now
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn times_called(&self) -> usize {
        self.times_called_async().join()
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// async_std::task::block_on(async {
    ///     // Arrange: Create mock server and a mock
    ///     use httpmock::{MockServer, Mock};
    ///
    ///     let mock_server = MockServer::start_async().await;
    ///
    ///     let mut mock = mock_server
    ///         .mock_async(|when, then| {
    ///             when.path("/times_called");
    ///             then.status(200);
    ///         })
    ///         .await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(mock_server.url("/times_called")).await.unwrap();
    ///
    ///     // Assert: Fetch how often this mock has been called from the server until now
    ///     assert_eq!(mock.times_called_async().await, 1);
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn times_called_async(&self) -> usize {
        let response = self
            .mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        response.call_counter
    }

    /// Deletes the associated mock object from the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mut mock = mock_server.mock(|when, then|{
    ///    when.path("/test");
    ///    then.status(202);
    /// });
    ///
    /// // Send a first request, then delete the mock from the mock and send another request.
    /// let response1 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// // Fetch how often this mock has been called from the server until now
    /// assert_eq!(mock.times_called(), 1);
    ///
    /// // Delete the mock from the mock server
    /// mock.delete();
    ///
    /// let response2 = isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response1.status(), 202);
    /// assert_eq!(response2.status(), 404);
    /// ```
    pub fn delete(&mut self) {
        self.delete_async().join();
    }

    /// Deletes this mock from the mock server. This method is the asynchronous equivalent of
    /// [MockRef::delete](struct.MockRef.html#method.delete).
    ///
    /// # Example
    /// ```
    /// async_std::task::block_on(async {
    ///     // Arrange
    ///     use httpmock::{MockServer, Mock};
    ///
    ///     let mock_server = MockServer::start_async().await;
    ///
    ///     let mut mock = mock_server
    ///       .mock_async(|when, then|{
    ///           when.path("/test");
    ///           then.status(202);
    ///       })
    ///       .await;
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get_async(mock_server.url("/test")).await.unwrap();
    ///
    ///     // Fetch how often this mock has been called from the server until now
    ///     assert_eq!(mock.times_called_async().await, 1);
    ///
    ///     // Delete the mock from the mock server
    ///     mock.delete_async().await;
    ///
    ///     let response2 = isahc::get_async(mock_server.url("/test")).await.unwrap();
    ///
    ///     // Assert
    ///     assert_eq!(response1.status(), 202);
    ///     assert_eq!(response2.status(), 404);
    /// });
    /// ```
    pub async fn delete_async(&self) {
        self.mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .delete_mock(self.id)
            .await
            .expect("could not delete mock from server");
    }

    /// Returns the address of the mock server where the associated mock object is store on.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    ///
    /// println!("{}", mock_server.address());
    /// // Will print "127.0.0.1:12345",
    /// // where 12345 is the port that the mock server is running on.
    /// ```
    pub fn server_address(&self) -> &SocketAddr {
        self.mock_server.server_adapter.as_ref().unwrap().address()
    }
}

impl Mock {
    /// Creates a new mock that automatically returns HTTP status code 200 if hit by an HTTP call.
    pub fn new() -> Self {
        Mock {
            mock: MockDefinition {
                request: RequestRequirements {
                    method: None,
                    path: None,
                    path_contains: None,
                    headers: None,
                    header_exists: None,
                    cookies: None,
                    cookie_exists: None,
                    body: None,
                    json_body: None,
                    json_body_includes: None,
                    body_contains: None,
                    path_matches: None,
                    body_matches: None,
                    query_param_exists: None,
                    query_param: None,
                    matchers: None,
                },
                response: MockServerHttpResponse {
                    status: None,
                    headers: None,
                    body: None,
                    delay: None,
                },
            },
        }
    }

    /// Sets the expected URL path.
    /// * `path` - The URL path.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path("/test")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_path(mut self, path: &str) -> Self {
        self.mock.request.path = Some(path.to_string());
        self
    }

    /// Sets an substring that the URL path needs to contain.
    /// * `substring` - The substring to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path_contains("es")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_path_contains(mut self, substring: &str) -> Self {
        if self.mock.request.path_contains.is_none() {
            self.mock.request.path_contains = Some(Vec::new());
        }

        self.mock
            .request
            .path_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());

        self
    }

    /// Sets a regex that the URL path needs to match.
    /// * `regex` - The regex to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use regex::Regex;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path_matches(Regex::new("le$").unwrap())
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// isahc::get(mock_server.url("/example")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_path_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.path_matches.is_none() {
            self.mock.request.path_matches = Some(Vec::new());
        }

        self.mock
            .request
            .path_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets the expected HTTP method.
    ///
    /// * `method` - The HTTP method.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_method(GET)
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// isahc::get(mock_server.url("/")).unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_method(mut self, method: Method) -> Self {
        self.mock.request.method = Some(method.to_string());
        self
    }

    /// Sets the expected HTTP header.
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    /// * `value` - The header value.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_header("Authorization", "token 1234567890")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(BTreeMap::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets the requirement that the HTTP request needs to contain a specific header
    /// (value is unchecked, refer to [Mock::expect_header](struct.Mock.html#method.expect_header)).
    ///
    /// * `name` - The HTTP header name (header names are case-insensitive by RFC 2616).
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_header_exists("Authorization")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_header_exists(mut self, name: &str) -> Self {
        if self.mock.request.header_exists.is_none() {
            self.mock.request.header_exists = Some(Vec::new());
        }

        self.mock
            .request
            .header_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name.
    /// * `value` - The expected cookie value.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_cookie("SESSIONID", "1234567890")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_cookie(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.cookies.is_none() {
            self.mock.request.cookies = Some(BTreeMap::new());
        }

        self.mock
            .request
            .cookies
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    ///
    /// * `name` - The cookie name
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_cookie_exists("SESSIONID")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_cookie_exists(mut self, name: &str) -> Self {
        if self.mock.request.cookie_exists.is_none() {
            self.mock.request.cookie_exists = Some(Vec::new());
        }

        self.mock
            .request
            .cookie_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());
        self
    }

    /// Sets the required HTTP request body content.
    ///
    /// * `body` - The required HTTP request body.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::prelude::*;
    ///
    /// let mock_server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_body("The Great Gatsby")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// Request::post(&format!("http://{}/test", mock_server.address()))
    ///     .body("The Great Gatsby")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    pub fn expect_body(mut self, body: &str) -> Self {
        self.mock.request.body = Some(body.to_string());
        self
    }

    /// Sets the expected JSON body. This method expects a serializable serde object
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use httpmock::Method::POST;
    /// use serde_json::json;
    /// use isahc::prelude::*;
    ///
    /// // This is a temporary type that we will use for this test
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/users")
    ///     .expect_header("Content-Type", "application/json")
    ///     .expect_json_body_obj(&TestUser {
    ///         name: String::from("Fred"),
    ///     })
    ///     .return_status(201)
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/users", mock_server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!(&TestUser {
    ///         name: "Fred".to_string()
    ///     }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_json_body_obj<'a, T>(self, body: &T) -> Self
    where
        T: Serialize + Deserialize<'a>,
    {
        let json_value = serde_json::to_value(body).expect("Cannot serialize json body to JSON");
        self.expect_json_body(json_value)
    }

    /// Sets the expected JSON body. This method expects a [serde_json::Value](../serde_json/enum.Value.html)
    /// that will be serialized/deserialized to/from a JSON string.
    ///
    /// Note that this method does not set the `Content-Type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::POST;
    /// use serde_json::json;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/users")
    ///     .expect_header("Content-Type", "application/json")
    ///     .expect_json_body(json!({ "name": "Hans" }))
    ///     .return_status(201)
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/users", mock_server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!({ "name": "Hans" }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_json_body(mut self, body: Value) -> Self {
        self.mock.request.json_body = Some(body);
        self
    }

    /// Sets the expected partial JSON body.
    ///
    /// **Attention: The partial string needs to be a valid JSON string. It must contain
    /// the full object hierarchy from the original JSON object but can leave out irrelevant
    /// attributes (see example).**
    ///
    /// Note that this method does not set the `Content-Type` header automatically, so you
    /// need to provide one yourself!
    ///
    /// String format and attribute order are irrelevant.
    ///
    /// * `partial_body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ## Example
    /// Suppose your application sends the following JSON request body:
    /// ```json
    /// {
    ///     "parent_attribute" : "Some parent data goes here",
    ///     "child" : {
    ///         "target_attribute" : "Example",
    ///         "other_attribute" : "Another value"
    ///     }
    /// }
    /// ```
    /// If we only want to verify that `target_attribute` has value `Example` without the need
    /// to provive a full JSON object, we can use this method as follows:
    /// ```
    /// use httpmock::{MockServer, Mock};
    ///
    /// let mock_server = MockServer::start();
    /// let mut mock = Mock::new()
    ///     .expect_json_body_partial(r#"
    ///         {
    ///             "child" : {
    ///                 "target_attribute" : "Example"
    ///             }
    ///          }
    ///     "#)
    ///     .return_status(202)
    ///     .create_on(&mock_server);
    /// ```
    /// Please note that the JSON partial contains the full object hierachy, i.e. it needs to start
    /// from the root! It leaves out irrelevant attributes, however (`parent_attribute`
    /// and `child.other_attribute`).
    pub fn expect_json_body_partial(mut self, partial_body: &str) -> Self {
        if self.mock.request.json_body_includes.is_none() {
            self.mock.request.json_body_includes = Some(Vec::new());
        }

        let value =
            Value::from_str(partial_body).expect("cannot convert JSON string to serde value");

        self.mock
            .request
            .json_body_includes
            .as_mut()
            .unwrap()
            .push(value);
        self
    }

    /// Sets the expected HTTP body substring.
    ///
    /// * `substring` - The substring that will matched against.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock, Regex};
    /// use httpmock::Method::POST;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/books")
    ///     .expect_body_contains("Ring")
    ///     .return_status(201)
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let response = Request::post(mock_server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_body_contains(mut self, substring: &str) -> Self {
        if self.mock.request.body_contains.is_none() {
            self.mock.request.body_contains = Some(Vec::new());
        }

        self.mock
            .request
            .body_contains
            .as_mut()
            .unwrap()
            .push(substring.to_string());
        self
    }

    /// Sets a [Regex](type.Regex.html) for the expected HTTP body.
    ///
    /// * `regex` - The regex that the HTTP request body will matched against.
    ///
    /// ```
    /// use isahc::prelude::*;
    /// use httpmock::Method::POST;
    /// use httpmock::{MockServer, Mock, Regex};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/books")
    ///     .expect_body_matches(Regex::new("Fellowship").unwrap())
    ///     .return_status(201)
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let response = Request::post(mock_server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_body_matches(mut self, regex: Regex) -> Self {
        if self.mock.request.body_matches.is_none() {
            self.mock.request.body_matches = Some(Vec::new());
        }

        self.mock
            .request
            .body_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex));
        self
    }

    /// Sets a query parameter that needs to be provided.
    /// * `name` - The query parameter name that will matched against.
    /// * `value` - The value parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_query_param("query", "Metallica")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// get(mock_server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_query_param(mut self, name: &str, value: &str) -> Self {
        if self.mock.request.query_param.is_none() {
            self.mock.request.query_param = Some(BTreeMap::new());
        }

        self.mock
            .request
            .query_param
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets a query parameter that needs to exist in an HTTP request.
    /// * `name` - The query parameter name that will matched against.
    ///
    /// ```
    /// // Arrange
    /// use isahc::get;
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_query_param_exists("query")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// get(mock_server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_query_param_exists(mut self, name: &str) -> Self {
        if self.mock.request.query_param_exists.is_none() {
            self.mock.request.query_param_exists = Some(Vec::new());
        }

        self.mock
            .request
            .query_param_exists
            .as_mut()
            .unwrap()
            .push(name.to_string());

        self
    }

    /// Sets a custom function that will evaluate if an HTTP request matches custom matching rules.
    /// **Attention: This will NOT work in standalone mock server (search the docs for "Standalone"
    /// to get more information on the standalone mode).**
    /// * `request_matcher` - The matcher function.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_match(|req: MockServerRequest| {
    ///         req.path.ends_with("st")
    ///     })
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/test")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn expect_match(mut self, request_matcher: MockMatcherFunction) -> Self {
        if self.mock.request.matchers.is_none() {
            self.mock.request.matchers = Some(Vec::new());
        }

        self.mock
            .request
            .matchers
            .as_mut()
            .unwrap()
            .push(request_matcher);

        self
    }

    /// Sets the HTTP response code that will be returned by the mock server.
    ///
    /// * `status` - The status code.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_path("/hello")
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn return_status(mut self, status: usize) -> Self {
        self.mock.response.status = Some(status as u16);
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `body` - The response body content.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, MockServerRequest};
    /// use isahc::ResponseExt;
    ///
    /// // Arrange
    /// let mock_server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_path("/hello")
    ///     .return_status(200)
    ///     .return_body("ohi!")
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn return_body(mut self, body: &str) -> Self {
        self.mock.response.body = Some(body.to_string());
        self
    }

    /// Sets the JSON body for the HTTP response that will be returned by the mock server.
    ///
    /// The provided JSON object needs to be both, a deserializable and serializable serde object.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you need
    /// to provide one yourself!
    ///
    /// * `body` -  The HTTP response body the mock server will return in the form of a
    ///             serde_json::Value object.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use serde_json::{Value, json};
    /// use isahc::ResponseExt;
    /// use isahc::prelude::*;
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_path("/user")
    ///     .return_status(200)
    ///     .return_header("Content-Type", "application/json")
    ///     .return_json_body(json!({ "name": "Hans" }))
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/user")).unwrap();
    ///
    /// let user: Value =
    ///     serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// assert_eq!(user.as_object().unwrap().get("name").unwrap(), "Hans");
    /// ```
    pub fn return_json_body(mut self, body: Value) -> Self {
        self.mock.response.body = Some(body.to_string());
        self
    }

    /// Sets the JSON body that will be returned by the mock server.
    /// This method expects a serializable serde object that will be serialized/deserialized
    /// to/from a JSON string.
    ///
    /// Note that this method does not set the "Content-Type" header automatically, so you
    /// need to provide one yourself!
    ///
    /// * `body` - The HTTP body object that will be serialized to JSON using serde.
    ///
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use isahc::ResponseExt;
    ///
    /// // This is a temporary type that we will use for this example
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_path("/user")
    ///     .return_status(201)
    ///     .return_header("Content-Type", "application/json")
    ///     .return_json_body_obj(&TestUser {
    ///         name: String::from("Hans"),
    ///     })
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/user")).unwrap();
    ///
    /// let user: TestUser =
    ///     serde_json::from_str(&response.text().unwrap()).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(user.name, "Hans");
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn return_json_body_obj<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        let json_body =
            serde_json::to_value(body).expect("cannot serialize json body to JSON string ");
        self.return_json_body(json_body)
    }

    /// Sets an HTTP header that the mock server will return.
    ///
    /// * `name` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// ## Example
    /// You can use this method conveniently as follows:
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    /// use serde_json::Value;
    /// use isahc::ResponseExt;
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .return_status(200)
    ///     .return_header("Expires", "Wed, 21 Oct 2050 07:28:00 GMT")
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let mut response = isahc::get(mock_server.url("/")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(m.times_called(), 1);
    /// ```
    pub fn return_header(mut self, name: &str, value: &str) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }

        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(name.to_string(), value.to_string());

        self
    }

    /// Sets the HTTP response up to return a permanent redirect.
    ///
    /// In detail, this method will add the following information to the HTTP response:
    /// - A "Location" header with the provided URL as its value.
    /// - Status code will be set to 301 (if no other status code was set before).
    /// - The response body will be set to "Moved Permanently" (if no other body was set before).
    ///
    /// Further information: https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
    /// and https://tools.ietf.org/html/rfc2616#section-10.3.8.
    ///
    /// * `redirect_url` - THe URL to redirect to.
    ///
    /// ## Example
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    /// use isahc::ResponseExt;
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let redirect_mock = Mock::new()
    ///     .expect_path("/redirectPath")
    ///     .return_permanent_redirect("http://www.google.com")
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    ///
    /// let mut response = isahc::get(mock_server.url("/redirectPath")).unwrap();
    /// let body = response.text().unwrap();
    ///
    /// // Assert
    /// assert_eq!(redirect_mock.times_called(), 1);
    ///
    /// // Attention!: Note that all of these values are automatically added to the response
    /// // (see details in mock builder method documentation).
    /// assert_eq!(response.status(), 302);
    /// assert_eq!(body, "Found");
    /// assert_eq!(response.headers().get("Location").unwrap().to_str().unwrap(), target_url);
    /// ```
    pub fn return_permanent_redirect(mut self, redirect_url: &str) -> Self {
        // see https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
        if self.mock.response.status.is_none() {
            self = self.return_status(301);
        }
        if self.mock.response.body.is_none() {
            self = self.return_body("Moved Permanently");
        }
        self.return_header("Location", redirect_url)
    }

    /// Sets the HTTP response up to return a temporary redirect.
    ///
    /// In detail, this method will add the following information to the HTTP response:
    /// - A "Location" header with the provided URL as its value.
    /// - Status code will be set to 302 (if no other status code was set before).
    /// - The response body will be set to "Found" (if no other body was set before).
    ///
    /// Further information: https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
    /// and https://tools.ietf.org/html/rfc2616#section-10.3.8.
    ///
    /// * `redirect_url` - THe URL to redirect to.
    ///
    /// ## Example
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    /// use isahc::ResponseExt;
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let redirect_mock = Mock::new()
    ///     .expect_path("/redirectPath")
    ///     .return_temporary_redirect("http://www.google.com")
    ///     .create_on(&mock_server);
    ///
    /// // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    ///
    /// let mut response = isahc::get(mock_server.url("/redirectPath")).unwrap();
    /// let body = response.text().unwrap();
    ///
    /// // Assert
    /// assert_eq!(redirect_mock.times_called(), 1);
    ///
    /// // Attention!: Note that all of these values are automatically added to the response
    /// // (see details in mock builder method documentation).
    /// assert_eq!(response.status(), 302);
    /// assert_eq!(body, "Found");
    /// assert_eq!(response.headers().get("Location").unwrap().to_str().unwrap(), target_url);
    /// ```
    pub fn return_temporary_redirect(mut self, redirect_url: &str) -> Self {
        // see https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
        if self.mock.response.status.is_none() {
            self = self.return_status(302);
        }
        if self.mock.response.body.is_none() {
            self = self.return_body("Found");
        }
        self.return_header("Location", redirect_url)
    }

    /// Sets a duration that will delay the mock server response.
    ///
    /// * `duration` - The delay.
    ///
    /// ```
    /// // Arrange
    /// use std::time::{SystemTime, Duration};
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let start_time = SystemTime::now();
    /// let delay = Duration::from_secs(3);
    ///
    /// let mock_server = MockServer::start();
    ///
    /// let mock = Mock::new()
    ///     .expect_path("/delay")
    ///     .return_with_delay(delay)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(mock.times_called(), 1);
    /// assert_eq!(start_time.elapsed().unwrap() > delay, true);
    /// ```
    pub fn return_with_delay(mut self, duration: Duration) -> Self {
        self.mock.response.delay = Some(duration);
        self
    }

    /// This method creates the mock object at the [MockServer](struct.MockServer.html).
    /// It returns a [MockRef](struct.MockRef.html) object representing the reference of the
    /// created mock at the server. Only after the call of this method completes, the mock server
    /// will start serving HTTP requests as specified in the [Mock](struct.Mock.html) instance.
    ///
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    ///
    /// let _ = env_logger::try_init();
    /// let mock_server = MockServer::start();
    ///
    /// let mock = Mock::new()
    ///     .return_status(200)
    ///     .create_on(&mock_server);
    ///
    /// // Act
    /// let response = isahc::get(mock_server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// assert_eq!(mock.times_called(), 1);
    /// ```
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub fn create_on<'a>(self, mock_server: &'a MockServer) -> MockRef<'a> {
        self.create_on_async(mock_server).join()
    }

    /// This method creates the mock object at the [MockServer](struct.MockServer.html).
    /// It returns a [MockRef](struct.MockRef.html) object representing the reference of the
    /// created mock at the server. Only after the call of this method completes, the mock server
    /// will start serving HTTP requests as specified in the [Mock](struct.Mock.html) instance.
    ///
    /// ```
    /// async_std::task::block_on(async {
    ///     use std::time::{SystemTime, Duration};
    ///     use httpmock::{MockServer, Mock};
    ///     use tokio_test::block_on;
    ///     let _ = env_logger::try_init();
    ///
    ///     // Arrange
    ///     let mock_server = MockServer::start_async().await;
    ///
    ///     let mock = Mock::new()
    ///         .return_status(200)
    ///         .create_on_async(&mock_server)
    ///         .await;
    ///
    ///     // Act
    ///     let response = isahc::get_async(mock_server.url("/delay")).await.unwrap();
    ///
    ///     // Assert
    ///     assert_eq!(mock.times_called_async().await, 1);
    /// });
    /// ```
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub async fn create_on_async<'a>(self, mock_server: &'a MockServer) -> MockRef<'a> {
        let response = mock_server
            .server_adapter
            .as_ref()
            .unwrap()
            .create_mock(&self.mock)
            .await
            .expect("Cannot deserialize mock server response");
        MockRef {
            id: response.mock_id,
            mock_server,
        }
    }
}

impl Default for Mock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::Mock;

    /// This test makes sure that a mock has a successful response code (200) by default.
    #[test]
    fn fill_mock_requirements() {
        let mock = Mock::default();
        assert_eq!(mock.mock.response.status, 200);
    }
}
