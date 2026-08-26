// import React from "react";
// import ReactDOM from "react-dom/client";
// import App from "./App";
//
// ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
//   <React.StrictMode>
//     <App />
//   </React.StrictMode>,
// );
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import {
    createHashHistory,
    RouterProvider,
    createRouter,
} from "@tanstack/react-router";

// Import the generated route tree
import { routeTree } from "./routeTree.gen";

// Create a hashHistory
const hashHistory = createHashHistory();

// Create a new router instance
const router = createRouter({
    routeTree,
    history: hashHistory,

    // 1. Enable View Transitions
    defaultViewTransition: {
        types: ({ fromLocation, toLocation }) => {
            // If going to a note, use 'popup'
            if (toLocation.pathname.includes("/note/")) {
                return ["popup-open"];
            }
            if (fromLocation?.pathname.includes("/note/")){
                return ["popup-close"]
            }

            // 1. Get the history index (Tauri/Browser state)
            const fromIndex = fromLocation?.state.__TSR_index || 0;
            const toIndex = toLocation.state.__TSR_index || 0;

            // 2. Decide direction
            if (toIndex > fromIndex) {
                return ["slide-left"]; // Going forward
            } else {
                return ["slide-right"]; // Going backward
            }
        },
    },
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
    interface Register {
        router: typeof router;
    }
}

// Render the app
const rootElement = document.getElementById("root")!;
if (!rootElement.innerHTML) {
    const root = ReactDOM.createRoot(rootElement);
    root.render(
        <StrictMode>
            <RouterProvider router={router} />
        </StrictMode>,
    );
}
