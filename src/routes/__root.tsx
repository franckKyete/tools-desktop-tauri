import * as React from "react";
import {
    Outlet,
    createRootRoute,
    // useRouterState,
} from "@tanstack/react-router";
// import { AnimatePresence, motion } from "framer-motion";
import "../root.css";

export const Route = createRootRoute({
    component: RootComponent,
});

function RootComponent() {
    // 1. Hook into the router state to get the current location
    // This ensures we have a unique key for every page change
    // const location = useRouterState({ select: (s) => s.location });
    return (
        <React.Fragment>
            {/* <AnimatePresence mode="wait"> */}
            {/*     <motion.div */}
            {/*         key={location.pathname} // <--- CRITICAL: This forces the re-render */}
            {/*         initial={{ opacity: 0, y: 0 }} */}
            {/*         animate={{ opacity: 1, y: 0 }} */}
            {/*         exit={{ opacity: 0, y: 0 }} */}
            {/*         transition={{ duration: 0.2 }} */}
            {/*         className="page-content" */}
            {/*     > */}
            {/*         <Outlet /> */}
            {/*     </motion.div> */}
            {/* </AnimatePresence> */}
            <Outlet />
        </React.Fragment>
    );
}
