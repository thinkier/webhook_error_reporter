use std::backtrace::Backtrace;
use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;

use webhook_error_reporter::webhook_error_reporter_core::error::ReportableError;

#[derive(Debug)]
struct CustomError {
	pub message: String,
	pub trace: Backtrace,
}

impl Error for CustomError {
	fn backtrace(&self) -> Option<&Backtrace> {
		Some(&self.trace)
	}
}

impl Display for CustomError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}", self.message)
	}
}

#[webhook_report_error]
async fn yikes_on_bikes<'a, F: Fn() -> &'a str>(f: F) -> Result<(), ReportableError<Box<dyn Error>>> {
	let message = async {
		f().to_string()
	}.await;

	let err = CustomError {
		message,
		trace: Backtrace::capture(),
	};

	Err(Box::new(err))?;

	Ok(())
}

#[test]
pub fn control() {
	env::var("REPORT_ERRORS_AT").unwrap();
}

#[tokio::test]
pub async fn complex_integration() {
	yikes_on_bikes(|| {
		"A quick brown fox jumps over the lazy dog."
	}).await;
}
