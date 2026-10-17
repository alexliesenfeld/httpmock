//! Compiles response templates at registration and renders them against each matched request.

use std::collections::BTreeMap;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use minijinja::{AutoEscape, Environment, UndefinedBehavior, context};

use crate::common::data::{HttpMockRequest, HttpMockResponse, MockServerHttpResponse};

/// Server-owned compiled templates. Sharing these does not borrow the user's template strings.
pub(crate) struct ResponseTemplates {
    environment: Environment<'static>,
    has_body: bool,
    headers: Vec<(String, String)>,
}

impl ResponseTemplates {
    pub(crate) fn compile(response: &MockServerHttpResponse) -> Result<Self, minijinja::Error> {
        let mut environment = Environment::new();
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.set_auto_escape_callback(|_| AutoEscape::None);
        environment.set_keep_trailing_newline(true);

        // Bound template execution on the request-handling thread. No filesystem loader or
        // application callbacks are installed; templates can access only their request context.
        environment.set_fuel(Some(1_000_000));

        if let Some(source) = &response.body_template {
            environment.add_template_owned("body", source.clone())?;
        }

        let mut headers = Vec::new();
        for (index, (name, source)) in response.header_templates.iter().flatten().enumerate() {
            let template_name = format!("header[{index}] ({name})");
            environment.add_template_owned(template_name.clone(), source.clone())?;
            headers.push((name.clone(), template_name));
        }

        Ok(Self {
            environment,
            has_body: response.body_template.is_some(),
            headers,
        })
    }

    pub(crate) fn render(
        &self,
        request: &HttpMockRequest,
        response: &mut HttpMockResponse,
    ) -> Result<(), minijinja::Error> {
        let context = context!(request => request_context(request));

        if self.has_body {
            let body = self.environment.get_template("body")?.render(&context)?;
            response.body = Some(body.into());
        }

        for (name, template_name) in &self.headers {
            let value = self.environment.get_template(template_name)?.render(&context)?;
            response
                .headers
                .get_or_insert_with(Vec::new)
                .push((name.clone(), value));
        }

        Ok(())
    }
}

fn request_context(request: &HttpMockRequest) -> serde_json::Value {
    let headers = collect_values(
        request
            .headers_vec()
            .iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value.clone())),
    );

    let mut context = serde_json::json!({
        "method": request.method_str(),
        "path": request.uri().path(),
        "uri": request.uri_str(),
        "query": collect_values(request.query_params()),
        "headers": headers,
        "body": String::from_utf8_lossy(request.body_ref()),
        "body_base64": BASE64.encode(request.body_ref()),
    });

    // Non-JSON requests can still use every other field. Accessing an absent `request.json`
    // fails under strict mode, or the template can handle it with `is defined` / `default`.
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(request.body_ref()) {
        context["json"] = json;
    }

    context
}

fn collect_values(pairs: impl IntoIterator<Item = (String, String)>) -> BTreeMap<String, Vec<String>> {
    let mut values = BTreeMap::<String, Vec<String>>::new();
    for (name, value) in pairs {
        values.entry(name).or_default().push(value);
    }
    values
}
