// src/main.tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import PopupWindow from "./PopupWindow";
import "./global.css";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { ThemeProvider } from "./components/theme-provider";
import SettingsWindow from "./pages/Settings";
import Updater from "./pages/Updater";
import Consent from "./pages/Consent";

const Main =  () => {
  return( 
    <ThemeProvider
      defaultTheme={"system"}
    >
      <Router basename="/">
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/popup" element={<PopupWindow />} />
          <Route path="/settings" element={<SettingsWindow />} />
          <Route path="/updater" element={<Updater />} />
          <Route path="/consent" element={<Consent />} />
        </Routes>
      </Router>
    </ThemeProvider>
  )
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <div className="border-2 border-zinc-800 h-screen w-screen">
      <Main />
    </div>
  </React.StrictMode>
)
