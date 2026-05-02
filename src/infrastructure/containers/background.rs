// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::Arc;

use glib::MainContext;

use crate::infrastructure::containers::error::ContainerError;
use crate::infrastructure::logging::app_logger::AppLogger;

const LOG_DOMAIN: &str = concat!(env!("APP_ID"), ".background");

/// Run a blocking call on a worker thread and deliver the result to the GTK
/// main thread via a callback.
///
/// `R` is any `Send + Sync + ?Sized` type (e.g., `dyn IContainerUseCase`,
/// `dyn IContainerDriver`). GTK widgets must only be touched on the main
/// thread; all blocking I/O must not run there. This helper bridges the two.
pub fn spawn_driver_task<R, T, F, C>(resource: Arc<R>, task: F, callback: C)
where
    R: Send + Sync + ?Sized + 'static,
    T: Send + 'static,
    F: FnOnce(&R) -> Result<T, ContainerError> + Send + 'static,
    C: Fn(Result<T, ContainerError>) + 'static,
{
    let (tx, rx) = async_channel::bounded(1);

    let log = AppLogger::new(LOG_DOMAIN);
    log.debug("Spawning driver task on worker thread");

    std::thread::spawn(move || {
        log.debug("Worker thread started");
        let result = task(resource.as_ref());
        if let Err(ref e) = result {
            log.warning(&format!("Driver task error: {e:?}"));
        }
        let _ = tx.send_blocking(result);
    });

    let log2 = AppLogger::new(LOG_DOMAIN);
    MainContext::default().spawn_local(async move {
        match rx.recv().await {
            Ok(result) => {
                log2.debug("Result delivered to GTK main loop");
                callback(result);
            }
            Err(e) => {
                // The sender was dropped before sending — the worker thread panicked
                // or the channel was closed without a result. This should never happen
                // under normal operation.
                log2.critical(&format!("Channel recv failed — task result lost: {e}"));
            }
        }
    });
}
