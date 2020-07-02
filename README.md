# Webhook error reporter

Reports your rust errors to a webhook to your favourite chat service.

## Instructions
1. Get a slack-compatible webhook URL for your chat service.
    - For slack, this is just a webhook
    - For discord, this is `{YOUR_WEBHOOK}/slack`
2. Stick it in an environment variable.
    - Default is `REPORT_ERRORS_AT`, however, you can customize this in the proc macro declaration like this: `#[webhook_report_error("YOUR_ENVAR")]`
3. Make sure your function has a return type of `Result<_, ReportableError<dyn Error>` (Don't worry, you can just `.into()` it.)
4. Tack on the proc macro attribute `#[webhook_report_error]`
5. Paint it red! Make a mess!
6. Now, on top of your application being in deep s\*\*t, your chat channel, the one that takes the webhooks, is also a bleeping mess. Delightful!

## Example
```rust
#![feature(async_closure)]
#![feature(backtrace)]

#[tokio::main]
async fn main<E: Error>() -> Result<(), ReportableError<E>> {
    Err(/* "Paint it red" */)
}
```

## Runtime Dependencies
- You need nightly rust that has:
    - Futures
    - Async closures
    - Backtrace
- You'll need an executor for the futures (ie. tokio)

## Caveats
Discord imposes a 2000 character limit per message, as such, you'll need to enable the crate feature `split_msg`.

Have fun with the errors!