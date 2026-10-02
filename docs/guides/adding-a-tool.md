# Adding a New Tool to Desktop

This tutorial walks through creating a new modular capability ("Tool") within **Tools Desktop** (`tools-desktop-tauri`), from wire protocol definition to the React UI.

---

## Step-by-Step Tutorial

### Step 1: Define the Packet Payload ([`src-tauri/src/bridge/packets.rs`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/bridge/packets.rs))

1. Define a new payload struct with `serde`:
   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct QuickSnippetPayload {
       pub title: String,
       pub snippet: String,
   }
   ```

2. Add your payload variant to the [`Payload`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/bridge/packets.rs#L67-L76) enum:
   ```rust
   #[derive(Serialize, Deserialize, Debug, Clone)]
   #[serde(tag = "type", content = "payload")]
   pub enum Payload {
       Ping,
       Pong,
       Clipboard(ClipboardPayload),
       FileRequest(FileRequestPayload),
       FileResponse(FileResponsePayload),
       Error(ErrorPayload),
       Text(TextPayload),
       QuickSnippet(QuickSnippetPayload), // <--- New tool payload
   }
   ```

3. Declare the capability in [`BridgeManager::get_advertisement()`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/bridge/bridge_manager.rs#L151-L167):
   ```rust
   capabilities: vec![
       "text".to_string(),
       "websocket".to_string(),
       "bluetooth".to_string(),
       "quick-snippets".to_string(), // <--- Advertise capability
   ],
   ```

---

### Step 2: Handle Incoming Packets ([`src-tauri/src/bridge/bridge.rs`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/bridge/bridge.rs))

Update [`Bridge::handle_message()`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/bridge/bridge.rs#L244-L269) to process the incoming payload:

```rust
match packet.payload {
    Payload::QuickSnippet(snippet) => {
        println!("Received snippet from mobile: {}", snippet.title);
        EVENT_MANAGER.emit(SnippetReceivedEvent { snippet }).await;
    }
    // ... existing handlers ...
}
```

---

### Step 3: Implement Local Storage (Optional)

If your tool requires persistence:
1. Implement the `Storable` trait for your data structure:
   ```rust
   use crate::storage::storable::Storable;

   #[derive(Serialize, Deserialize, Debug, Clone)]
   pub struct Snippet {
       pub id: String,
       pub store_id: Option<i64>,
       pub content: String,
   }

   #[async_trait]
   impl Storable for Snippet {
       fn get_name() -> String { "Snippet".to_string() }
       fn get_store_id(&self) -> Option<i64> { self.store_id }
       fn serialize(&self) -> Vec<u8> { postcard::to_allocvec(self).unwrap() }
       fn deserialize(_: i64, data: &Vec<u8>) -> Result<Box<Self>, Box<dyn std::error::Error>> {
           Ok(Box::new(postcard::from_bytes(data)?))
       }
       async fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
           let id = Storage::store(self)?;
           self.store_id = Some(id);
           Ok(())
       }
   }
   ```

---

### Step 4: Expose Tauri IPC Commands ([`src-tauri/src/lib.rs`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src-tauri/src/lib.rs))

1. Write the command function:
   ```rust
   #[tauri::command]
   async fn send_snippet_to_mobile(title: String, snippet: String) -> Result<(), String> {
       let packet = Packet::new(Payload::QuickSnippet(QuickSnippetPayload { title, snippet }));
       let bridge_manager = BRIDGE_MANAGER.clone();
       // Forward to active bridges
       Ok(())
   }
   ```

2. Register it in the invoke handler:
   ```rust
   .invoke_handler(tauri::generate_handler![
       greet,
       get_advertisement,
       get_notes,
       send_snippet_to_mobile, // <--- Add your command
   ])
   ```

---

### Step 5: Build the Frontend View ([`src/routes/`](file:///home/kyete/kitchen/tools-ws/workspaces/main/tools-desktop-tauri/src/routes/))

Create a new file-based route in `@tanstack/react-router` (e.g. `src/routes/snippets.tsx`):

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/snippets")({
  component: SnippetsView,
});

function SnippetsView() {
  const handleSend = async () => {
    await invoke("send_snippet_to_mobile", {
      title: "API Key",
      snippet: "sk-...",
    });
  };

  return (
    <div className="p-8">
      <h1 className="text-2xl font-bold">Snippets</h1>
      <Button onClick={handleSend}>Send to Mobile</Button>
    </div>
  );
}
```
