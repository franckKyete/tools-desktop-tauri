
use std::future::Future;
use std::mem; // Added for std::mem::forget
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::UnixStream;
use zbus::interface;
use zbus::zvariant::{OwnedFd, OwnedObjectPath};

use tokio::runtime::Handle;


type FutureReturn = Pin<Box<dyn Future<Output = ()> + Send>>;
type NewConnectionCallback = Arc<dyn Fn(UnixStream) -> FutureReturn + Send + Sync + 'static>;


/// The structure implementing the BlueZ Profile1 D-Bus interface.
/// It holds a handle to the Tokio runtime to correctly spawn connection handlers.
pub struct Profile {
    runtime_handle: Handle,
    callback: NewConnectionCallback,
}

impl Profile {
    /// Create a new Profile instance, passing the main Tokio runtime handle.
    // pub fn new<F>(runtime_handle: Handle, new_connection: F) -> Self
    // where
    //     F: Fn(UnixStream) + Send + Sync + 'static,
    pub fn new<F, Fut>(runtime_handle: Handle, new_connection: F) -> Self
    where
        F: Fn(UnixStream) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Profile {
            runtime_handle,
            callback : Arc::new(move |stream| Box::pin(new_connection(stream)) ),
        }
    }
}

#[interface(name = "org.bluez.Profile1")]
impl Profile {
    fn release(&self) {
        println!("Profile released");
    }

    fn new_connection(
        &self,
        _device: OwnedObjectPath,
        fd: OwnedFd,
        _props: std::collections::HashMap<String, zbus::zvariant::Value>,
    ) {
        println!("Incoming connection");

        let raw_fd = fd.as_raw_fd();
        mem::forget(fd);
        let rt_handle = self.runtime_handle.clone();

        let callback = self.callback.clone();

        rt_handle.spawn(async move {
            println!(
                "Successfully obtained raw_fd. Starting stream conversion inside Tokio task..."
            );

            // --- Conversion now happens *inside* the tokio task ---
            let stream = unsafe {
                // SAFETY: We assume this FD refers to a Unix socket stream.

                // 1. Create a standard blocking UnixStream, taking ownership of the raw_fd.
                let std_stream = std::os::unix::net::UnixStream::from_raw_fd(raw_fd);

                // 2. Convert to non-blocking tokio stream.
                // This call now has access to the runtime context (reactor).
                match UnixStream::from_std(std_stream) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error converting std stream to tokio stream: {}", e);
                        return;
                    }
                }
            };
            callback(stream).await;

            println!("Stream established. Starting handler logic...");
        });
    }
}

