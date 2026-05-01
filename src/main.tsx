import React from "react";
import ReactDOM from "react-dom/client";

import { PreferencesPage } from "./features/preferences/PreferencesPage";
import { WarningToast } from "./features/warning/WarningToast";
import { BreakOverlay } from "./features/overlay/BreakOverlay";
import "./index.css";

type Route = "preferences" | "warning" | "overlay";

function resolveRoute(): Route {
  const param = new URLSearchParams(window.location.search).get("route");
  switch (param) {
    case "warning":
      return "warning";
    case "overlay":
      return "overlay";
    default:
      return "preferences";
  }
}

function Root() {
  const route = resolveRoute();
  switch (route) {
    case "warning":
      return <WarningToast />;
    case "overlay":
      return <BreakOverlay />;
    case "preferences":
    default:
      return <PreferencesPage />;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
