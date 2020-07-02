use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;

#[derive(Debug)]
pub struct ReportableError<E> {
	cause: E,
	pub reporter_error: Option<ReporterError>,
}

impl<E: Error> Display for ReportableError<E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if let Some(re) = &self.reporter_error {
			writeln!(f, "failed to report the error: {}", self.cause)?;
			write!(f, "reporter failed to report the error: {}", re)
		} else {
			write!(f, "successfully reported: {}", self.cause)
		}
	}
}

impl<E: Error + 'static> From<E> for ReportableError<E> {
	fn from(cause: E) -> Self {
		ReportableError {
			cause,
			reporter_error: None,
		}
	}
}

impl<E: Error + 'static> Error for ReportableError<E> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.cause)
	}

	fn backtrace(&self) -> Option<&Backtrace> {
		self.cause.backtrace()
	}
}

#[derive(Debug)]
pub enum ReporterError {
	InvalidUrl(String),
	SerializationError(serde_json::Error),
	HyperError(hyper::Error),
	ServerError(u16, String),
	ServerResponseNotUtf8(u16),
	#[cfg(feature = "split_msg")]
	MsgSplitFail,
}

impl Display for ReporterError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		let msg = match self {
			Self::InvalidUrl(url) => format!("invalid url: {}", url),
			Self::SerializationError(se) => format!("error while serialization: {}", se),
			Self::HyperError(he) => format!("hyper error: {}", he),
			Self::ServerError(http_code, server_msg) => format!("server returned an unokay http code {}, message: {}", http_code, server_msg),
			Self::ServerResponseNotUtf8(http_code) => format!("server returned an unokay http code {}, however, message is not utf-8", http_code),
			#[cfg(feature = "split_msg")]
			Self::MsgSplitFail => format!("reporter error: embedded message has a line exceeding 2k characters, splitting failed.")
		};

		write!(f, "{}", msg)
	}
}

impl Error for ReporterError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		if let Self::HyperError(e) = &self {
			return Some(e);
		}

		None
	}

	fn backtrace(&self) -> Option<&Backtrace> {
		if let Self::HyperError(e) = &self {
			return e.backtrace();
		}

		None
	}
}

