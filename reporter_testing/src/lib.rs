#![feature(async_closure)]
#![feature(backtrace)]

#[cfg(test)]
extern crate tokio;
#[cfg(test)]
#[macro_use]
extern crate webhook_error_reporter;

#[cfg(test)]
mod tests;
