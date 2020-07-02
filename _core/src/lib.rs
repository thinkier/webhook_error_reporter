#![feature(backtrace)]
extern crate hyper;
extern crate hyper_tls;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::backtrace::BacktraceStatus;
use std::error::Error;

use hyper::{Body, Client, Request, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use crate::error::{ReportableError, ReporterError};

pub mod error;

const BACKTRACE_NOT_SUPPORTED: &str = "(Backtrace is not supported.)";
const BACKTRACE_NOT_IMPLEMENTED: &str = "(Backtrace is not implemented by the error type.)";
const BACKTRACE_DISABLED: &str = "RUST_BACKTRACE must be set to `1` or `full` for the backtrace to be captured.";

pub async fn report(cause: Box<dyn Error>, webhook: &str) -> ReportableError<Box<dyn Error>> {
	let fut = async {
		let https = HttpsConnector::new();
		let client = Client::builder().build::<_, hyper::Body>(https);

		let webhook_uri = webhook.parse::<Uri>().map_err(|_| {
			ReporterError::InvalidUrl(webhook.to_string())
		})?;

		let err_backtrace = if let Some(bt) = cause.backtrace() {
			match bt.status() {
				BacktraceStatus::Captured => format!("Backtrace:\n{}", bt),
				BacktraceStatus::Disabled => BACKTRACE_DISABLED.to_string(),
				BacktraceStatus::Unsupported => BACKTRACE_NOT_SUPPORTED.to_string(),
				_ => BACKTRACE_NOT_SUPPORTED.to_string()
			}
		} else {
			BACKTRACE_NOT_IMPLEMENTED.to_string()
		};

		let buf = format!("Error caught, message: {}\n\nBacktrace: {}\n", cause, err_backtrace);

		#[cfg(not(feature = "split_msg"))]
			{
				send_fragment(&buf, &webhook_uri, &client).await?;
			}
		#[cfg(feature = "split_msg")]
			{
				let mut frag = String::new();
				for line in buf.lines() {
					// Hard breakage (with problems)
					if line.len() >= 2000 {
						return Err(ReporterError::MsgSplitFail);
					}

					// Soft breakage
					if frag.len() + line.len() < 2000 {
						frag += "\n";
						frag += line;
					} else {
						send_fragment(&frag, &webhook_uri, &client).await?;
						frag.clear();
						frag += line;
					}
				}

				if frag.len() > 0 {
					send_fragment(&frag, &webhook_uri, &client).await?;
				}
			}

		Ok(())
	}.await;

	let mut wrapper: ReportableError<_> = cause.into();

	if let Err(re) = fut {
		wrapper.reporter_error = Some(re);
	}

	return wrapper;
}

async fn send_fragment(frag: &str, webhook_uri: &Uri, client: &Client<HttpsConnector<HttpConnector>>) -> Result<(), ReporterError> {
	let req = Request::builder()
		.method("POST")
		.uri(webhook_uri)
		.header("Content-Type", "application/json")
		.body(Body::from(serde_json::to_string(&json!({
				"text": frag
			})).map_err(|se| {
			ReporterError::SerializationError(se)
		})?))
		.expect("failed to build post request");

	let resp = client.request(req).await
		.map_err(|he| {
			ReporterError::HyperError(he)
		})?;

	let code = resp.status().as_u16();
	// If it's
	// HTTP 200 OK,
	// HTTP 201 Created,
	// HTTP 202 Accepted,
	// HTTP 203 Non-Authoritative Information
	// HTTP 204 No Content.
	if !(200 <= code && code <= 204) {
		let bytes = hyper::body::to_bytes(resp.into_body()).await
			.map_err(|he| ReporterError::HyperError(he))?;

		let str = String::from_utf8(bytes.to_vec())
			.map_err(|_| ReporterError::ServerResponseNotUtf8(code))?;

		return Err(ReporterError::ServerError(code, str));
	}

	Ok(())
}