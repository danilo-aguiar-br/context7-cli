// SPDX-License-Identifier: MIT OR Apache-2.0
//! Binary entry point for `context7`.
//!
//! Wires together the platform init, logging, i18n resolution, and the
//! async CLI dispatcher in [`context7_cli::run`]. Translates any
//! returned [`context7_cli::errors::Context7Error`] into a BSD-style
//! exit code (see [`context7_cli::errors::Context7Error::exit_code`]).

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    context7_cli::platform::init_platform();

    let _log_guard = match context7_cli::init_logging() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Error: failed to initialise logging: {e}");
            std::process::exit(74); // EX_IOERR
        }
    };

    if let Err(e) = context7_cli::run().await {
        // Extract BSD-style exit code from Context7Error if available
        let code = e
            .downcast_ref::<context7_cli::errors::Context7Error>()
            .map(|ec| ec.exit_code())
            .unwrap_or(1);

        // KeysOperationFailed already printed a user-friendly message; skip empty msgs
        let msg = e.to_string();
        if !msg.is_empty() {
            eprintln!("Error: {e:#}");
        }
        std::process::exit(code);
    }
}
