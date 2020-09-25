## Version 0.5.0
- _**Breaking Change**_: Function `Mock::expect_json_body` was renamed to `expect_json_body_obj`.
- _**Breaking Change**_: Function `Mock::return_json_body` was renamed to `return_json_body_obj`.
- _**Attention**: A new API for mock definition was added. The old API is still available. No changed required!_
- Most API methods now accept convertible types (such as `Into<String>`) instead of concrete types (such as `&str`).
- Function `Mock::return_body` and `Then::body` now accept a `Into<Vec<u8>>` parameter instead of `String`. No changes required on your side! 
  This allows for binary content in the response body.  
- A new function `expect_json_body` was added which takes a `serde_json::Value` as an argument.
- A new function `return_json_body` was added which takes a `serde_json::Value` as an argument.
- Improved documentation (a lot!).
- Debug log output is now pretty printed!
- Cookie matching support.
- Support for convenient temporary and permanent redirect.
- The log level of some log messages was changed from `debug` to `trace` to make debugging easier.

## Version 0.4.5
- Improved documentation.
- Added a new function `base_url` to the `MockServer` structure.
