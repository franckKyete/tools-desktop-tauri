// #[derive(Clone)]
// pub enum Event {
//     NewConnection(String),
//     MessageUpdate(String),
//     Disconnected(String),
//     Reconnected(String),
//     Clipboard(String),
// }
//
// #[derive(Eq, Hash, PartialEq)]
// pub enum EventName {
//     NewConnection,
//     MessageUpdate,
//     Disconnected,
//     Clipboard,
// }
//
// impl Event {
//     pub fn name(&self) -> EventName {
//         match self {
//             Event::NewConnection(_) => EventName::NewConnection,
//             Event::MessageUpdate(_) => EventName::MessageUpdate,
//             Event::Disconnected(_) => EventName::Disconnected,
//             Event::Reconnected(_) => EventName::Disconnected,
//             Event::Clipboard(_) => EventName::Clipboard,
//         }
//     }
// }

// This struct is your event. It must derive Clone.
#[derive(Clone, Debug)]
pub struct DisconnectedEvent {
    pub id : String
}
#[derive(Clone, Debug)]
pub struct ConnectedEvent {
    pub id: String,
}
#[derive(Clone, Debug)]
pub struct ClipboardEvent {
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct ReconnectedEvent {
    pub id: String,
}
#[derive(Clone, Debug)]
pub struct MessageUpdateEvent {
    pub id: String,
}

