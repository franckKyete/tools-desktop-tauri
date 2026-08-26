use async_trait::async_trait;
use lazy_static::lazy_static;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};

use tokio::sync::{Mutex, mpsc};

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

pub struct EventManager {
    callbacks: Mutex<CallbacksMap>,
    // The channel now sends a tuple of the TypeId and the event payload
    to_event_manager: Mutex<mpsc::UnboundedSender<(EventKey, EventPayload)>>,
    from_anywhere: Mutex<mpsc::UnboundedReceiver<(EventKey, EventPayload)>>,
}



impl EventManager {
    pub fn new() -> Arc<Self> {
        // The channel type is updated
        let (to_event_manager, from_anywhere) =
            mpsc::unbounded_channel::<(EventKey, EventPayload)>();
        Arc::new(Self {
            to_event_manager: Mutex::new(to_event_manager),
            from_anywhere: Mutex::new(from_anywhere),
            callbacks: Mutex::new(HashMap::new()),
        })
    }

    /// Emits a type-safe event.
    ///
    /// The event type T must be 'static + Send + Sync.
    /// The compiler can infer T from the 'event' parameter.
    pub async fn emit<T: Any + Send + Sync + 'static>(self: &Arc<Self>, event: T) {
        let key = TypeId::of::<T>();
        let payload = Arc::new(event) as EventPayload; // Wrap in Arc

        let to_event_manager = self.to_event_manager.lock().await;
        // Send both the key and the payload
        to_event_manager.send((key, payload)).unwrap();
    }

    /// Runs the event manager loop.
    pub fn run(self: &Arc<Self>) {
        let this = self.clone();
        tokio::task::spawn(async move {
            // Destructure the tuple from the channel
            while let Some((key, payload)) = this.from_anywhere.lock().await.recv().await {
                println!("New event ");
                let callbacks_lock = this.callbacks.lock().await;
                if let Some(callbacks) = callbacks_lock.get(&key) {
                    for callback in callbacks.iter() {
                        // Clone the Arc for each callback
                        println!("Calling callback");
                        tokio::spawn({
                            let callback = callback.clone();
                            let payload = payload.clone();
                            async move {
                                callback(payload.clone()).await;
                            }
                        });
                        // callback(payload.clone()).await;
                    }
                }
                println!("That's good");
            }
            println!("Not this");
        });
    }

    /// Registers a type-safe event listener.
    ///
    /// The event type T must be 'static + Send + Sync + Clone.
    /// The callback 'F' must take T (by value) as its parameter.
    pub async fn on<T, F, Fut>(self: &Arc<Self>, callback: F)
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

// lazy_static! {
//     pub static ref EVENT_MANAGER: Arc<EventManager> = {
//         let em = EventManager::new();
//         em.run(); // Start the event loop immediately
//         em
//     };
// }
// use lazy_static::lazy_static;
// use std::{collections::HashMap, pin::Pin, sync::Arc};
//
// use tokio::sync::{Mutex, mpsc};
//
// use super::{Event, EventName};
//
// type FutureReturn = Pin<Box<dyn Future<Output = ()> + Send>>;
// type Callbacks = Vec<Arc<dyn Fn(Event) -> FutureReturn + Send + Sync>>;
// pub struct EventManager {
//     callbacks: Mutex<HashMap<EventName, Callbacks>>,
//     to_event_manager: Mutex<mpsc::UnboundedSender<Event>>,
//     from_anywhere: Mutex<mpsc::UnboundedReceiver<Event>>,
// }
//
//
// impl EventManager {
//     pub fn new() -> Arc<Self> {
//         let (to_event_manager, from_anywhere) = mpsc::unbounded_channel::<Event>();
//         Arc::new(Self {
//             to_event_manager: Mutex::new(to_event_manager),
//             from_anywhere: Mutex::new(from_anywhere),
//             callbacks: Mutex::new(HashMap::new()),
//         })
//     }
//
//     pub async fn emit(self: &Arc<Self>, event: Event) {
//         let to_event_manager = self.to_event_manager.lock().await;
//         to_event_manager.send(event).unwrap();
//     }
//
//     pub fn run(self: &Arc<Self>) {
//         let this = self.clone();
//         tokio::task::spawn(async move {
//             while let Some(event) = this.from_anywhere.lock().await.recv().await {
//                 let callbacks_lock = this.callbacks.lock().await;
//                 if let Some(callbacks) = callbacks_lock.get(&event.name()) {
//                     for callback in callbacks.iter() {
//                         callback(event.clone()).await;
//                     }
//                 }
//             }
//         });
//     }
//
//     pub async fn on<F, Fut>(self: &Arc<Self>, event: EventName, callback: F)
//     where
//         F: Fn(Event) -> Fut + Send + Sync + 'static,
//         Fut: Future<Output = ()> + Send + 'static,
//     {
//         let mut callbacks_lock = self.callbacks.lock().await;
//         callbacks_lock
//             .entry(event)
//             .or_default()
//             .push(Arc::new(move |event| Box::pin(callback(event))));
//
//     }
// }
//
lazy_static! {
    pub static ref EVENT_MANAGER: Arc<EventManager> = EventManager::new();
}
