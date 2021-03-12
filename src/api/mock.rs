use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[cfg(feature = "color")]
use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{Method, Regex};
use crate::data::{
    ActiveMock, ClosestMatch, HttpMockRequest, MockDefinition, MockMatcherFunction,
    MockServerHttpResponse, Pattern, RequestRequirements,
};
use crate::server::{Diff, DiffResult, Mismatch, Reason, Tokenizer};
use crate::util::{get_test_resource_file_path, read_file, Join};
use crate::MockServer;

/// Represents a reference to the mock object on a [MockServer](struct.MockServer.html).
/// It can be used to spy on the mock and also perform some management operations, such as
/// deleting the mock from the [MockServer](struct.MockServer.html).
///
/// # Example
/// ```
/// // Arrange
/// use httpmock::{MockServer, Mock};
///
/// let server = MockServer::start();
///
/// let mut mock = server.mock(|when, then|{
///    when.path("/test");
///    then.status(202);
/// });
///
/// // Send a first request, then delete the mock from the mock and send another request.
/// let response1 = isahc::get(server.url("/test")).unwrap();
///
/// // Fetch how often this mock has been called from the server until now
/// assert_eq!(mock.hits(), 1);
///
/// // Delete the mock from the mock server
/// mock.delete();
///
/// let response2 = isahc::get(server.url("/test")).unwrap();
///
/// // Assert
/// assert_eq!(response1.status(), 202);
/// assert_eq!(response2.status(), 404);
/// ```
pub struct MockRef<'a> {
    // Please find the reason why id is public in
    // https://github.com/alexliesenfeld/httpmock/issues/26.
    pub id: usize,
    server: &'a MockServer,
}

impl<'a> MockRef<'a> {
    pub fn new(id: usize, server: &'a MockServer) -> Self {
        Self { id, server }
    }
    /// This method asserts that the mock server received **exactly one** HTTP request that matched
    /// all the request requirements of this mock.
    ///
    /// **Attention**: If you want to assert more than one request, consider using either
    /// [MockRef::assert_hits](struct.MockRef.html#method.assert_hits) or
    /// [MockRef::hits](struct.MockRef.html#method.hits).
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// isahc::get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock server received exactly one request that matched all
    /// // the request requirements of the mock.
    /// mock.assert();
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn assert(&self) {
        self.assert_async().join()
    }

    /// This method asserts that the mock server received **exactly one** HTTP request that matched
    /// all the request requirements of this mock.
    ///
    /// **Attention**: If you want to assert more than one request, consider using either
    /// [MockRef::assert_hits](struct.MockRef.html#method.assert_hits) or
    /// [MockRef::hits](struct.MockRef.html#method.hits).
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    ///  async_std::task::block_on(async {
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server.mock_async(|when, then| {
    ///         when.path("/hits");
    ///         then.status(200);
    ///     }).await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock server received exactly one request that matched all
    ///     // the request requirements of the mock.
    ///     mock.assert_async().await;
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn assert_async(&self) {
        self.assert_hits_async(1).await
    }

    /// This method asserts that the mock server received the provided number of HTTP requests which
    /// matched all the request requirements of this mock.
    ///
    /// **Attention**: Consider using the shorthand version
    /// [MockRef::assert](struct.MockRef.html#method.assert) if you want to assert only one hit.
    ///
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    /// use isahc::get;
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// get(server.url("/hits")).unwrap();
    /// get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock server received exactly two requests that matched all
    /// // the request requirements of the mock.
    /// mock.assert_hits(2);
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn assert_hits(&self, hits: usize) {
        self.assert_hits_async(hits).join()
    }

    /// This method asserts that the mock server received the provided number of HTTP requests which
    /// matched all the request requirements of this mock.
    ///
    /// **Attention**: Consider using the shorthand version
    /// [MockRef::assert_async](struct.MockRef.html#method.assert_async) if you want to assert only one hit.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    ///  async_std::task::block_on(async {
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server.mock_async(|when, then| {
    ///         when.path("/hits");
    ///         then.status(200);
    ///     }).await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock server received exactly two requests that matched all
    ///     // the request requirements of the mock.
    ///     mock.assert_hits_async(2).await;
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn assert_hits_async(&self, hits: usize) {
        let active_mock = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        if active_mock.call_counter == hits {
            return;
        }

        if active_mock.call_counter > hits {
            assert_eq!(
                active_mock.call_counter, hits,
                "The number of matching requests was higher than expected (expected {} but was {})",
                hits, active_mock.call_counter
            )
        }

        let closest_match = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .verify(&active_mock.definition.request)
            .await
            .expect("Cannot contact mock server");

        fail_with(active_mock.call_counter, hits, closest_match)
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange: Create mock server and a mock
    /// use httpmock::{MockServer, Mock};
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then| {
    ///     when.path("/hits");
    ///     then.status(200);
    /// });
    ///
    /// // Act: Send a request, then delete the mock from the mock and send another request.
    /// isahc::get(server.url("/hits")).unwrap();
    ///
    /// // Assert: Make sure the mock has been called exactly one time
    /// assert_eq!(1, mock.hits());
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub fn hits(&self) -> usize {
        self.hits_async().join()
    }

    /// This method returns the number of times a mock has been called at the mock server.
    /// Deprecated, use [Mock::hits](struct.MockServer.html#method.hits) instead.
    #[deprecated(since = "0.5.0", note = "Please use 'hits' function instead")]
    pub fn times_called(&self) -> usize {
        self.hits()
    }

    /// This method returns the number of times a mock has been called at the mock server.
    ///
    /// # Example
    /// ```
    /// async_std::task::block_on(async {
    ///     // Arrange: Create mock server and a mock
    ///     use httpmock::{MockServer, Mock};
    ///
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server
    ///         .mock_async(|when, then| {
    ///             when.path("/hits");
    ///             then.status(200);
    ///         })
    ///         .await;
    ///
    ///     // Act: Send a request, then delete the mock from the mock and send another request.
    ///     isahc::get_async(server.url("/hits")).await.unwrap();
    ///
    ///     // Assert: Make sure the mock was called with all required attributes exactly one time.
    ///     assert_eq!(1, mock.hits_async().await);
    /// });
    /// ```
    /// # Panics
    /// This method will panic if there is a problem with the (standalone) mock server.
    pub async fn hits_async(&self) -> usize {
        let response = self
            .server
            .server_adapter
            .as_ref()
            .unwrap()
            .fetch_mock(self.id)
            .await
            .expect("cannot deserialize mock server response");

        response.call_counter
    }

    /// This method returns the number of times a mock has been called at the mock server.
    /// Deprecated, use [Mock::hits](struct.MockServer.html#method.hits_async) instead.
    #[deprecated(since = "0.5.0", note = "Please use 'hits_async' function instead")]
    pub async fn times_called_async(&self) -> usize {
        self.hits_async().await
    }

    /// Deletes the associated mock object from the mock server.
    ///
    /// # Example
    /// ```
    /// // Arrange
    /// use httpmock::{MockServer, Mock};
    ///
    /// let server = MockServer::start();
    ///
    /// let mut mock = server.mock(|when, then|{
    ///    when.path("/test");
    ///    then.status(202);
    /// });
    ///
    /// // Send a first request, then delete the mock from the mock and send another request.
    /// let response1 = isahc::get(server.url("/test")).unwrap();
    ///
    /// // Fetch how often this mock has been called from the server until now
    /// assert_eq!(mock.hits(), 1);
    ///
    /// // Delete the mock from the mock server
    /// mock.delete();
    ///
    /// let response2 = isahc::get(server.url("/test")).unwrap();
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
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mut mock = server
    ///       .mock_async(|when, then|{
    ///           when.path("/test");
    ///           then.status(202);
    ///       })
    ///       .await;
    ///
    ///     // Send a first request, then delete the mock from the mock and send another request.
    ///     let response1 = isahc::get_async(server.url("/test")).await.unwrap();
    ///
    ///     // Fetch how often this mock has been called from the server until now
    ///     assert_eq!(mock.hits_async().await, 1);
    ///
    ///     // Delete the mock from the mock server
    ///     mock.delete_async().await;
    ///
    ///     let response2 = isahc::get_async(server.url("/test")).await.unwrap();
    ///
    ///     // Assert
    ///     assert_eq!(response1.status(), 202);
    ///     assert_eq!(response2.status(), 404);
    /// });
    /// ```
    pub async fn delete_async(&self) {
        self.server
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
    /// let server = MockServer::start();
    ///
    /// println!("{}", server.address());
    /// // Will print "127.0.0.1:12345",
    /// // where 12345 is the port that the mock server is running on.
    /// ```
    pub fn server_address(&self) -> &SocketAddr {
        self.server.server_adapter.as_ref().unwrap().address()
    }
}

/// The [MockRefExt](trait.MockRefExt.html) trait extends the [MockRef](struct.MockRef.html)
/// structure with some additional functionality, that is usually not required.
pub trait MockRefExt<'a> {
    /// Creates a new [MockRef](struct.MockRef.html) instance that references an already existing
    /// mock on a [MockServer](struct.MockServer.html). This functionality is usually not required.
    /// You can use it if for you need to recreate [MockRef](struct.MockRef.html) instances
    ///.
    /// * `id` - The ID of the existing mock ot the [MockServer](struct.MockServer.html).
    /// * `mock_server` - The [MockServer](struct.MockServer.html) to which the
    /// [MockRef](struct.MockRef.html) instance will reference.
    ///
    /// # Example
    /// ```
    /// use httpmock::{MockServer, MockRef, MockRefExt};
    /// use isahc::get;
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let mock_ref = server.mock(|when, then| {
    ///     when.path("/test");
    ///     then.status(202);
    /// });
    ///
    /// // Store away the mock ID for later usage and drop the MockRef instance.
    /// let mock_id = mock_ref.id();
    /// drop(mock_ref);
    ///
    /// // Act: Send the HTTP request
    /// let response = get(server.url("/test")).unwrap();
    ///
    /// // Create a new MockRef instance that references the earlier mock at the MockServer.
    /// let mock_ref = MockRef::new(mock_id, &server);
    ///
    /// // Use the recreated MockRef as usual.
    /// mock_ref.assert();
    /// assert_eq!(response.status(), 202);
    /// ```
    /// Refer to [`Issue 26`][https://github.com/alexliesenfeld/httpmock/issues/26] for more
    /// information.
    fn new(id: usize, mock_server: &'a MockServer) -> MockRef<'a>;

    /// Returns the ID that the mock was assigned to on the
    /// [MockServer](struct.MockServer.html).
    fn id(&self) -> usize;
}

impl<'a> MockRefExt<'a> for MockRef<'a> {
    fn new(id: usize, mock_server: &'a MockServer) -> MockRef<'a> {
        MockRef {
            id,
            server: mock_server,
        }
    }

    fn id(&self) -> usize {
        self.id
    }
}

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
/// let server = MockServer::start();
///
/// let mock = Mock::new()
///     .expect_path_contains("/search")
///     .expect_query_param("query", "metallica")
///     .return_status(202)
///     .create_on(&server);
///
/// // Act: Send the HTTP request
/// let response = isahc::get(server.url("/search?query=metallica")).unwrap();
///
/// // Assert
/// mock.assert();
/// assert_eq!(response.status(), 202);
/// ```
/// Observe how [Mock::create_on](struct.Mock.html#method.create_on) is used to create a mock object
/// on the server. After the call completes, the mock server will start serving HTTP requests
/// as specified in the [Mock](struct.Mock.html) instance.
///
/// The [Mock::create_on](struct.Mock.html#method.create_on) method also returns a mock reference
/// that identifies the mock object on the server. It can be used to fetch related information
/// from the server, such as the number of times the mock was served
/// (see [MockRef::hits](struct.MockRef.html#method.hits)). You can also use it
/// to explicitly delete the mock object from the server
/// (see [MockRef::delete](struct.MockRef.html#method.delete)).
#[deprecated(
    since = "0.5.0",
    note = "Please use newer API (see: https://github.com/alexliesenfeld/httpmock/blob/master/CHANGELOG.md#version-050)"
)]
pub struct Mock {
    mock: MockDefinition,
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
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path("/test")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// isahc::get(server.url("/test")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_path<S: Into<String>>(mut self, path: S) -> Self {
        self.mock.request.path = Some(path.into());
        self
    }

    /// Sets an substring that the URL path needs to contain.
    /// * `substring` - The substring to match against.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path_contains("es")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// isahc::get(server.url("/test")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_path_contains<S: Into<String>>(mut self, substring: S) -> Self {
        if self.mock.request.path_contains.is_none() {
            self.mock.request.path_contains = Some(Vec::new());
        }

        self.mock
            .request
            .path_contains
            .as_mut()
            .unwrap()
            .push(substring.into());

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
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_path_matches(Regex::new("le$").unwrap())
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// isahc::get(server.url("/example")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_path_matches<R: Into<Regex>>(mut self, regex: R) -> Self {
        if self.mock.request.path_matches.is_none() {
            self.mock.request.path_matches = Some(Vec::new());
        }

        self.mock
            .request
            .path_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex.into()));
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
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_method(GET)
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// isahc::get(server.url("/")).unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_method<M: Into<Method>>(mut self, method: M) -> Self {
        self.mock.request.method = Some(method.into().to_string());
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
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_header("Authorization", "token 1234567890")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_header<S: Into<String>>(mut self, name: S, value: S) -> Self {
        if self.mock.request.headers.is_none() {
            self.mock.request.headers = Some(Vec::new());
        }

        self.mock
            .request
            .headers
            .as_mut()
            .unwrap()
            .push((name.into(), value.into()));

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
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_header_exists("Authorization")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Authorization", "token 1234567890")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_header_exists<S: Into<String>>(mut self, name: S) -> Self {
        if self.mock.request.header_exists.is_none() {
            self.mock.request.header_exists = Some(Vec::new());
        }

        self.mock
            .request
            .header_exists
            .as_mut()
            .unwrap()
            .push(name.into());
        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    /// **Attention**: Cookie names are **case-sensitive**.
    ///
    /// * `name` - The cookie name.
    /// * `value` - The expected cookie value.
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_cookie("SESSIONID", "1234567890")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_cookie<S: Into<String>>(mut self, name: S, value: S) -> Self {
        if self.mock.request.cookies.is_none() {
            self.mock.request.cookies = Some(Vec::new());
        }

        self.mock
            .request
            .cookies
            .as_mut()
            .unwrap()
            .push((name.into(), value.into()));

        self
    }

    /// Sets the cookie that needs to exist in the HTTP request.
    /// Cookie parsing follows [RFC-6265](https://tools.ietf.org/html/rfc6265.html).
    /// **Attention**: Cookie names are **case-sensitive**.
    ///
    /// * `name` - The cookie name
    ///
    /// # Example
    /// ```
    /// use httpmock::{Mock, MockServer};
    /// use httpmock::Method::GET;
    /// use regex::Regex;
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_cookie_exists("SESSIONID")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .header("Cookie", "TRACK=12345; SESSIONID=1234567890; CONSENT=1")
    ///     .body(())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_cookie_exists<S: Into<String>>(mut self, name: S) -> Self {
        if self.mock.request.cookie_exists.is_none() {
            self.mock.request.cookie_exists = Some(Vec::new());
        }

        self.mock
            .request
            .cookie_exists
            .as_mut()
            .unwrap()
            .push(name.into());
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
    /// use isahc::{prelude::*, Request};
    ///
    /// let server = MockServer::start();
    /// let mock = Mock::new()
    ///     .expect_body("The Great Gatsby")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// Request::post(&format!("http://{}/test", server.address()))
    ///     .body("The Great Gatsby")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// mock.assert();
    /// ```
    pub fn expect_body<S: Into<String>>(mut self, body: S) -> Self {
        self.mock.request.body = Some(body.into());
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
    /// use isahc::{prelude::*, Request};
    ///
    /// // This is a temporary type that we will use for this test
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/users")
    ///     .expect_header("Content-Type", "application/json")
    ///     .expect_json_body_obj(&TestUser {
    ///         name: String::from("Fred"),
    ///     })
    ///     .return_status(201)
    ///     .create_on(&server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/users", server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!(&TestUser {
    ///         name: "Fred".to_string()
    ///     }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
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
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/users")
    ///     .expect_header("Content-Type", "application/json")
    ///     .expect_json_body(json!({ "name": "Hans" }))
    ///     .return_status(201)
    ///     .create_on(&server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let mut response = Request::post(&format!("http://{}/users", server.address()))
    ///     .header("Content-Type", "application/json")
    ///     .body(json!({ "name": "Hans" }).to_string())
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn expect_json_body<V: Into<Value>>(mut self, body: V) -> Self {
        self.mock.request.json_body = Some(body.into());
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
    /// let server = MockServer::start();
    /// let mut mock = Mock::new()
    ///     .expect_json_body_partial(r#"
    ///         {
    ///             "child" : {
    ///                 "target_attribute" : "Example"
    ///             }
    ///          }
    ///     "#)
    ///     .return_status(202)
    ///     .create_on(&server);
    /// ```
    /// Please note that the JSON partial contains the full object hierachy, i.e. it needs to start
    /// from the root! It leaves out irrelevant attributes, however (`parent_attribute`
    /// and `child.other_attribute`).
    pub fn expect_json_body_partial<S: Into<String>>(mut self, partial_body: S) -> Self {
        if self.mock.request.json_body_includes.is_none() {
            self.mock.request.json_body_includes = Some(Vec::new());
        }

        let value = Value::from_str(&partial_body.into())
            .expect("cannot convert JSON string to serde value");

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
    /// use isahc::{prelude::*, Request};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/books")
    ///     .expect_body_contains("Ring")
    ///     .return_status(201)
    ///     .create_on(&server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let response = Request::post(server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn expect_body_contains<S: Into<String>>(mut self, substring: S) -> Self {
        if self.mock.request.body_contains.is_none() {
            self.mock.request.body_contains = Some(Vec::new());
        }

        self.mock
            .request
            .body_contains
            .as_mut()
            .unwrap()
            .push(substring.into());
        self
    }

    /// Sets a [Regex](type.Regex.html) for the expected HTTP body.
    ///
    /// * `regex` - The regex that the HTTP request body will matched against.
    ///
    /// ```
    /// use isahc::{prelude::*, Request};
    /// use httpmock::Method::POST;
    /// use httpmock::{MockServer, Mock, Regex};
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    ///
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_method(POST)
    ///     .expect_path("/books")
    ///     .expect_body_matches(Regex::new("Fellowship").unwrap())
    ///     .return_status(201)
    ///     .create_on(&server);
    ///
    /// // Act: Send the request and deserialize the response to JSON
    /// let response = Request::post(server.url("/books"))
    ///     .body("The Fellowship of the Ring")
    ///     .unwrap()
    ///     .send()
    ///     .unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// ```
    pub fn expect_body_matches<R: Into<Regex>>(mut self, regex: R) -> Self {
        if self.mock.request.body_matches.is_none() {
            self.mock.request.body_matches = Some(Vec::new());
        }

        self.mock
            .request
            .body_matches
            .as_mut()
            .unwrap()
            .push(Pattern::from_regex(regex.into()));
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
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_query_param("query", "Metallica")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// // Act
    /// get(server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// ```
    pub fn expect_query_param<S: Into<String>>(mut self, name: S, value: S) -> Self {
        if self.mock.request.query_param.is_none() {
            self.mock.request.query_param = Some(Vec::new());
        }

        self.mock
            .request
            .query_param
            .as_mut()
            .unwrap()
            .push((name.into(), value.into()));

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
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_query_param_exists("query")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// // Act
    /// get(server.url("/search?query=Metallica")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// ```
    pub fn expect_query_param_exists<S: Into<String>>(mut self, name: S) -> Self {
        if self.mock.request.query_param_exists.is_none() {
            self.mock.request.query_param_exists = Some(Vec::new());
        }

        self.mock
            .request
            .query_param_exists
            .as_mut()
            .unwrap()
            .push(name.into());

        self
    }

    /// Sets a custom function that will evaluate if an HTTP request matches custom matching rules.
    /// **Attention: This will NOT work in standalone mock server (search the docs for "Standalone"
    /// to get more information on the standalone mode).**
    /// * `request_matcher` - The matcher function.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock, HttpMockRequest};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_match(|req: &HttpMockRequest| {
    ///         req.path.ends_with("st")
    ///     })
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// // Act
    /// let response = isahc::get(server.url("/test")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
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
    /// use httpmock::{MockServer, Mock};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_path("/hello")
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// // Act
    /// let response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn return_status(mut self, status: u16) -> Self {
        self.mock.response.status = Some(status);
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `body` - The response body content.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_path("/hello")
    ///     .return_status(200)
    ///     .return_body("ohi!")
    ///     .create_on(&server);
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// ```
    pub fn return_body(mut self, body: impl AsRef<[u8]>) -> Self {
        self.mock.response.body = Some(body.as_ref().to_vec());
        self
    }

    /// Sets the HTTP response body that will be returned by the mock server.
    ///
    /// * `resource_file_path` - The file path to the file with the response body content. The file
    ///                          path can either be absolute or relative to the project root
    ///                         directory.
    ///
    /// ## Example:
    /// ```
    /// use httpmock::{MockServer, Mock};
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // Arrange
    /// let server = MockServer::start();
    /// let m = Mock::new()
    ///     .expect_path("/hello")
    ///     .return_status(200)
    ///     .return_body_from_file("tests/resources/simple_body.txt")
    ///     .create_on(&server);
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/hello")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(response.text().unwrap(), "ohi!");
    /// ```
    pub fn return_body_from_file<S: Into<String>>(mut self, resource_file_path: S) -> Self {
        let resource_file_path = resource_file_path.into();
        let path = Path::new(&resource_file_path);
        let absolute_path = match path.is_absolute() {
            true => path.to_path_buf(),
            false => get_test_resource_file_path(&resource_file_path).expect(&format!(
                "Cannot create absolute path from string '{}'",
                &resource_file_path
            )),
        };
        let content = read_file(&absolute_path).expect(&format!(
            "Cannot read from file {}",
            absolute_path.to_str().expect("Invalid OS path")
        ));
        self.return_body(content)
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
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_path("/user")
    ///     .return_status(200)
    ///     .return_header("Content-Type", "application/json")
    ///     .return_json_body(json!({ "name": "Hans" }))
    ///     .create_on(&server);
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/user")).unwrap();
    ///
    /// let user: Value =
    ///     serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// assert_eq!(user.as_object().unwrap().get("name").unwrap(), "Hans");
    /// ```
    pub fn return_json_body<V: Into<Value>>(mut self, body: V) -> Self {
        self.mock.response.body = Some(body.into().to_string().into_bytes());
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
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// // This is a temporary type that we will use for this example
    /// #[derive(serde::Serialize, serde::Deserialize)]
    /// struct TestUser {
    ///     name: String,
    /// }
    ///
    /// // Arrange
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .expect_path("/user")
    ///     .return_status(201)
    ///     .return_header("Content-Type", "application/json")
    ///     .return_json_body_obj(&TestUser {
    ///         name: String::from("Hans"),
    ///     })
    ///     .create_on(&server);
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/user")).unwrap();
    ///
    /// let user: TestUser =
    ///     serde_json::from_str(&response.text().unwrap()).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 201);
    /// assert_eq!(user.name, "Hans");
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
    /// let server = MockServer::start();
    ///
    /// let m = Mock::new()
    ///     .return_status(200)
    ///     .return_header("Expires", "Wed, 21 Oct 2050 07:28:00 GMT")
    ///     .create_on(&server);
    ///
    /// // Act
    /// let mut response = isahc::get(server.url("/")).unwrap();
    ///
    /// // Assert
    /// m.assert();
    /// assert_eq!(response.status(), 200);
    /// ```
    pub fn return_header<S: Into<String>>(mut self, name: S, value: S) -> Self {
        if self.mock.response.headers.is_none() {
            self.mock.response.headers = Some(BTreeMap::new());
        }

        self.mock
            .response
            .headers
            .as_mut()
            .unwrap()
            .insert(name.into(), value.into());

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
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let redirect_mock = Mock::new()
    ///     .expect_path("/redirectPath")
    ///     .return_permanent_redirect("http://www.google.com")
    ///     .create_on(&server);
    ///
    /// // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    /// let mut response = isahc::get(server.url("/redirectPath")).unwrap();
    /// let body = response.text().unwrap();
    ///
    /// // Assert
    /// assert_eq!(redirect_mock.hits(), 1);
    ///
    /// // Attention!: Note that all of these values are automatically added to the response
    /// // (see details in mock builder method documentation).
    /// assert_eq!(response.status(), 301);
    /// assert_eq!(body, "Moved Permanently");
    /// assert_eq!(response.headers().get("Location").unwrap().to_str().unwrap(), "http://www.google.com");
    /// ```
    #[deprecated(
        since = "0.5.6",
        note = "Please use desired response code and headers instead"
    )]
    pub fn return_permanent_redirect<S: Into<String>>(mut self, redirect_url: S) -> Self {
        // see https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
        if self.mock.response.status.is_none() {
            self = self.return_status(301);
        }
        if self.mock.response.body.is_none() {
            self = self.return_body("Moved Permanently");
        }
        self.return_header("Location", &redirect_url.into())
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
    /// use isahc::{prelude::*, ResponseExt};
    ///
    /// let _ = env_logger::try_init();
    /// let server = MockServer::start();
    ///
    /// let redirect_mock = Mock::new()
    ///     .expect_path("/redirectPath")
    ///     .return_temporary_redirect("http://www.google.com")
    ///     .create_on(&server);
    ///
    /// // Act: Send the HTTP request with an HTTP client that DOES NOT FOLLOW redirects automatically!
    ///
    /// let mut response = isahc::get(server.url("/redirectPath")).unwrap();
    /// let body = response.text().unwrap();
    ///
    /// // Assert
    /// assert_eq!(redirect_mock.hits(), 1);
    ///
    /// // Attention!: Note that all of these values are automatically added to the response
    /// // (see details in mock builder method documentation).
    /// assert_eq!(response.status(), 302);
    /// assert_eq!(body, "Found");
    /// assert_eq!(response.headers().get("Location").unwrap().to_str().unwrap(), "http://www.google.com");
    /// ```
    #[deprecated(
        since = "0.5.6",
        note = "Please use desired response code and headers instead"
    )]
    pub fn return_temporary_redirect<S: Into<String>>(mut self, redirect_url: S) -> Self {
        // see https://developer.mozilla.org/en-US/docs/Web/HTTP/Redirections
        if self.mock.response.status.is_none() {
            self = self.return_status(302);
        }
        if self.mock.response.body.is_none() {
            self = self.return_body("Found");
        }
        self.return_header("Location", &redirect_url.into())
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
    /// let server = MockServer::start();
    ///
    /// let mock = Mock::new()
    ///     .expect_path("/delay")
    ///     .return_with_delay(delay)
    ///     .create_on(&server);
    ///
    /// // Act
    /// let response = isahc::get(server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// mock.assert();
    /// assert_eq!(start_time.elapsed().unwrap() > delay, true);
    /// ```
    pub fn return_with_delay<D: Into<Duration>>(mut self, duration: D) -> Self {
        self.mock.response.delay = Some(duration.into());
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
    /// let server = MockServer::start();
    ///
    /// let mock = Mock::new()
    ///     .return_status(200)
    ///     .create_on(&server);
    ///
    /// // Act
    /// let response = isahc::get(server.url("/delay")).unwrap();
    ///
    /// // Assert
    /// mock.assert();
    /// ```
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub fn create_on<'a>(self, server: &'a MockServer) -> MockRef<'a> {
        self.create_on_async(server).join()
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
    ///     let server = MockServer::start_async().await;
    ///
    ///     let mock = Mock::new()
    ///         .return_status(200)
    ///         .create_on_async(&server)
    ///         .await;
    ///
    ///     // Act
    ///     let response = isahc::get_async(server.url("/delay")).await.unwrap();
    ///
    ///     // Assert
    ///     mock.assert_async().await;
    /// });
    /// ```
    ///
    /// # Panics
    /// This method will panic if there is a problem communicating with the server.
    pub async fn create_on_async<'a>(self, server: &'a MockServer) -> MockRef<'a> {
        let response = server
            .server_adapter
            .as_ref()
            .unwrap()
            .create_mock(&self.mock)
            .await
            .expect("Cannot deserialize mock server response");
        MockRef {
            id: response.mock_id,
            server,
        }
    }
}

impl Default for Mock {
    fn default() -> Self {
        Self::new()
    }
}

fn create_reason_output(reason: &Reason) -> String {
    let mut output = String::new();
    let offsets = match reason.best_match {
        true => ("\t".repeat(5), "\t".repeat(2)),
        false => ("\t".repeat(1), "\t".repeat(2)),
    };
    let actual_text = match reason.best_match {
        true => "Actual (closest match):",
        false => "Actual:",
    };
    output.push_str(&format!(
        "Expected:{}[{}]\t\t{}\n",
        offsets.0, reason.comparison, &reason.expected
    ));
    output.push_str(&format!(
        "{}{}{}\t{}\n",
        actual_text,
        offsets.1,
        " ".repeat(reason.comparison.len() + 7),
        &reason.actual
    ));
    output
}

fn create_diff_result_output(dd: &DiffResult) -> String {
    let mut output = String::new();
    output.push_str("Diff:");
    if dd.differences.is_empty() {
        output.push_str("<empty>");
    }
    output.push_str("\n");

    dd.differences.iter().for_each(|d| {
        match d {
            Diff::Same(e) => {
                output.push_str(&format!("   | {}", e));
            }
            Diff::Add(e) => {
                #[cfg(feature = "color")]
                output.push_str(&format!("+++| {}", e).green().to_string());
                #[cfg(not(feature = "color"))]
                output.push_str(&format!("+++| {}", e));
            }
            Diff::Rem(e) => {
                #[cfg(feature = "color")]
                output.push_str(&format!("---| {}", e).red().to_string());
                #[cfg(not(feature = "color"))]
                output.push_str(&format!("---| {}", e));
            }
        }
        output.push_str("\n")
    });
    output.push_str("\n");
    output
}

fn create_mismatch_output(idx: usize, mm: &Mismatch) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} : {}", idx + 1, &mm.title));
    output.push_str("\n");
    output.push_str(&"-".repeat(90));
    output.push_str("\n");

    mm.reason
        .as_ref()
        .map(|reason| output.push_str(&create_reason_output(reason)));

    mm.diff
        .as_ref()
        .map(|diff_result| output.push_str(&create_diff_result_output(diff_result)));

    output.push_str("\n");
    output
}

fn fail_with(actual_hits: usize, expected_hits: usize, closest_match: Option<ClosestMatch>) {
    match closest_match {
        None => assert!(false, "No request has been received by the mock server."),
        Some(closest_match) => {
            let mut output = String::new();
            output.push_str(&format!(
                "{} of {} expected requests matched the mock specification, .\n",
                actual_hits, expected_hits
            ));
            output.push_str(&format!(
                "Here is a comparison with the most similar non-matching request (request number {}): \n\n",
                closest_match.request_index + 1
            ));

            for (idx, mm) in closest_match.mismatches.iter().enumerate() {
                output.push_str(&create_mismatch_output(idx, &mm));
            }

            closest_match.mismatches.first().map(|mismatch| {
                mismatch
                    .reason
                    .as_ref()
                    .map(|reason| assert_eq!(reason.expected, reason.actual, "{}", output))
            });

            assert!(false, output)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::api::mock::fail_with;
    use crate::data::{ClosestMatch, HttpMockRequest};
    use crate::server::{Diff, DiffResult, Mismatch, Reason, Tokenizer};
    use crate::Mock;

    #[test]
    #[should_panic(expected = "1 : This is a title\n\
    ------------------------------------------------------------------------------------------\n\
    Expected:	[equals]		/toast\n\
    Actual:		             	/test\n\
    Diff:\n   | t\n---| e\n+++| oa\n   | st")]
    fn fail_with_message_test() {
        // Arrange
        let closest_match = ClosestMatch {
            request: HttpMockRequest {
                path: "/test".to_string(),
                method: "GET".to_string(),
                headers: None,
                query_params: None,
                body: None,
            },
            request_index: 0,
            mismatches: vec![Mismatch {
                title: "This is a title".to_string(),
                reason: Some(Reason {
                    expected: "/toast".to_string(),
                    actual: "/test".to_string(),
                    comparison: "equals".to_string(),
                    best_match: false,
                }),
                diff: Some(DiffResult {
                    differences: vec![
                        Diff::Same(String::from("t")),
                        Diff::Rem(String::from("e")),
                        Diff::Add(String::from("oa")),
                        Diff::Same(String::from("st")),
                    ],
                    distance: 5,
                    tokenizer: Tokenizer::Line,
                }),
            }],
        };

        // Act
        fail_with(1, 2, Some(closest_match));

        // Assert
        // see "should panic" annotation
    }
}
