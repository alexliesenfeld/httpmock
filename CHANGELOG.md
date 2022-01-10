## Version 0.6.6
- Changed some API to allow for more type flexibility (see https://github.com/alexliesenfeld/httpmock/issues/58).
- Fixed parsing query parameter values that contain `+` to represent space (see https://github.com/alexliesenfeld/httpmock/issues/56).

## Version 0.6.5
- Fixes a race condition that could occur when deleting mocks from the mock server (see https://github.com/alexliesenfeld/httpmock/issues/53).
- Replaced internal diff library (switched from `difference` to `similar`, see https://github.com/alexliesenfeld/httpmock/pull/55). 

## Version 0.6.4
- Fixed minimum Rust version in README (raised from 1.47 to 1.54, see release 0.6.3 for more information).

## Version 0.6.3
- This is a maintenance release that updates all dependencies to the most recent version.
- Bumped minimum Rust version to 1.54 due to transitive dependency.

## Version 0.6.2
- A bug was fixed that has unexported the [When](https://docs.rs/httpmock/0.5.8/httpmock/struct.When.html) and
  [Then](https://docs.rs/httpmock/0.5.8/httpmock/struct.When.html) structures. Both types are now exported again.
  Please refer to https://github.com/alexliesenfeld/httpmock/issues/47 for more info.
  
## Version 0.6.1
- This is a maintenance release that updates all dependencies to the most recent version.

## Version 0.6.0
### General
- Old [Mock](https://docs.rs/httpmock/0.4.5/httpmock/struct.Mock.html) structure based API was deprecated 
  starting from version 0.5.0 and was removed with this version. Please switch to the new API based on the 
  [When](https://docs.rs/httpmock/0.5.8/httpmock/struct.When.html) / 
  [Then](https://docs.rs/httpmock/0.5.8/httpmock/struct.When.html) structures.
- The two methods `MockRef::times_called` and `MockRef::times_called_async` were deprecated since version 0.5.0 and 
  have now been removed.
- A [prelude module](https://github.com/alexliesenfeld/httpmock#getting-started) was added to shorten imports 
  that are usually required when using `httpmock` in tests.
- The struct `MockRef` has been renamed to `Mock`. 
- Trait `MockRefExt` has been renamed to `MockExt`.
- Added support for x-www-form-urlencoded request bodies.

### Standalone Mock Server
- Standalone server now has a request history limit that can be adjusted.
- All standalone servers parameters now have an environment variable fallback.
- Standalone servers `exposed` and `disable_access_log` parameters were changed, so that they now require a value 
  in addition to the flag itself (this is due to a limitation of `structopt`/`clap`): 
  Before: `httpmock --expose`, Now: `httpmock --expose true`.

## Version 0.5.8
- A bug has been fixed that prevented to use the mock server for requests containing a `multipart/form-data` 
  request body with binary data.

## Version 0.5.7
- Added static mock support based on YAML files for standalone mode.
- Dockerfile Rust version has been fixed.
- Documentation on query parameters has been enhanced.
- Bumped minimum Rust version to 1.46 due to transitive dependency.

## Version 0.5.6 
- A bug has been fixed that caused false positive warnings in the log output.
- Updated all dependencies to the most recent versions.
- Assertion error messages (`MockRef::assert` and `MockRef::assert_hits`) now contain more details.

## Version 0.5.5
- A bug has been fixed that prevented to use a request body in DELETE requests.  

## Version 0.5.4
- A new extension trait `MockRefExt` was added that extends the `MockRef` structure with additional but usually 
not required functionality.  

## Version 0.5.3
- This is a maintenance release that updates all dependencies to the most recent version.
- This release bumps the minimal Rust version from 1.43+ to 1.45+.

## Version 0.5.2
- Updated dependencies to newest version.
- Removed dependency version fixation from v0.5.1.
- `Mock::return_body_from_file` and `Then::body_from_file` now accept absolute and relative file paths.
 
## Version 0.5.1
- Updated dependency to futures-util to fix compile errors.
- Fixed all dependency version numbers to avoid future problems with new dependency version releases.

## Version 0.5.0
- ❌ _**Breaking Change**_: Function `Mock::expect_json_body` was renamed to `expect_json_body_obj`.
- ❌ _**Breaking Change**_: Function `Mock::return_json_body` was renamed to `return_json_body_obj`.
- 🚀 _**Attention**: A new API for mock definition was added. The old API is still available and functional, 
but is deprecated from now on. Please consider switching to the new API._
- 🚀 **Attention**: The following new assertion functions have been added that will provide you smart and helpful 
error output to support debugging:
    - `MockRef::assert`
    - `MockRef::assert_hits`
    - `MockRef::assert_async`
    - `MockRef::assert_hits_async`
- The two methods `MockRef::times_called` and `MockRef::times_called_async` are now deprecated. Consider using
`MockRef::hits` and `MockRef::hits_async`.
- The two methods `Mock::return_body` and `Then::body` now accept binary content.
- The following new methods accept a `serde_json::Value`:
    - `Mock::expect_json_body`
    - `Mock::return_json_body`
    - `When::json_body`
    - `Then::json_body`
- 🔥 Improved documentation (**a lot!**).
- 👏 Debug log output is now pretty printed! 
- 🍪 Cookie matching support.
- Support for convenient temporary and permanent redirect.
- The log level of some log messages was changed from `debug` to `trace` to make debugging easier.

## Version 0.4.5
- Improved documentation.
- Added a new function `base_url` to the `MockServer` structure.
