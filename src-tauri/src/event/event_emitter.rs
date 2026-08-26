use async_trait::async_trait;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};

use tokio::sync::Mutex;

#[async_trait]
pub trait TEventEmitter : Send + Sync {
    async fn emit<T: Any + Send + Sync + 'static>(&self, event: T);
    async fn on<T, F, Fut>(&self, callback: F)
    where
        T: Any + Send + Sync + Clone + 'static,  // Event must be Clone
        F: Fn(T) -> Fut + Send + Sync + 'static, // Callback takes T by value
        Fut: Future<Output = ()> + Send + 'static;
}

// --- Updated Type Aliases ---
// This is our type-erased event payload.
// We use Arc so we can clone it for multiple listeners.
type EventPayload = Arc<dyn Any + Send + Sync>;
// The key for our map is the TypeId of the event struct.
type EventKey = TypeId;
// The return type for our async callbacks.
type FutureReturn = Pin<Box<dyn Future<Output = ()> + Send>>;
// A type-erased callback. It takes the Arc<dyn Any> payload.
type ErasedCallback = Arc<dyn Fn(EventPayload) -> FutureReturn + Send + Sync>;
// The map storing all callbacks, keyed by EventKey (TypeId).
type CallbacksMap = HashMap<EventKey, Vec<ErasedCallback>>;

pub struct EventEmitter {
    callbacks: Mutex<CallbacksMap>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TEventEmitter for EventEmitter {
    /// Emits a type-safe event.
    ///
    /// The event type T must be 'static + Send + Sync.
    /// The compiler can infer T from the 'event' parameter.
    async fn emit<T: Any + Send + Sync + 'static>(&self, event: T) {
        let key = TypeId::of::<T>();
        let payload = Arc::new(event) as EventPayload; // Wrap in Arc

        let callbacks_lock = self.callbacks.lock().await;
        if let Some(callbacks) = callbacks_lock.get(&key) {
            for callback in callbacks.iter() {
                // Clone the Arc for each callback
                tokio::spawn({
                    let callback = callback.clone();
                    let payload = payload.clone();
                    async move {
                        callback(payload.clone()).await;
                    }
                });
            }
        }
    }

    /// Registers a type-safe event listener.
    ///
    /// The event type T must be 'static + Send + Sync + Clone.
    /// The callback 'F' must take T (by value) as its parameter.
    async fn on<T, F, Fut>(&self, callback: F)
    where
        T: Any + Send + Sync + Clone + 'static,  // Event must be Clone
        F: Fn(T) -> Fut + Send + Sync + 'static, // Callback takes T by value
        Fut: Future<Output = ()> + Send + 'static,
    {
        let key = TypeId::of::<T>();

        // Create a type-erased wrapper closure
        let erased_callback: ErasedCallback = Arc::new(move |payload: EventPayload| {
            // Downcast the Arc<dyn Any> payload back to &T
            match payload.downcast_ref::<T>() {
                Some(event_ref) => {
                    // Clone the event to pass it by value
                    let event_clone = event_ref.clone();
                    // Call the user's concrete callback
                    Box::pin(callback(event_clone))
                }
                None => {
                    // This should never happen if 'emit' is used correctly
                    eprintln!("Event manager logic error: TypeId mismatch during downcast.");
                    Box::pin(async {})
                }
            }
        });

        let mut callbacks_lock = self.callbacks.lock().await;
        callbacks_lock.entry(key).or_default().push(erased_callback);
    }
}
