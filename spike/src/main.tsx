import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

// No browser "Reload / Inspect" menu in the shipped app.
if (import.meta.env.PROD) document.addEventListener("contextmenu", (e) => e.preventDefault());

ReactDOM.createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
