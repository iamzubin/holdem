// src/main.tsx
import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import PopupWindow from "./PopupWindow";
import "./global.css";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { ThemeProvider } from "./components/theme-provider";
import { getCurrentWindow, Theme } from '@tauri-apps/api/window';

const Main =  () => {
  const window = getCurrentWindow();

  const [theme, setTheme] = useState<Theme>("dark");

  useEffect(() => {
    console.log("window.theme()", (window.theme()));
    window.theme().then((_theme) => {
      console.log("theme", _theme);
      setTheme(_theme as Theme);
    });
    console.log("theme_", theme);
  }, []);

  return( 

    <ThemeProvider
    defaultTheme={"system"}
  >
      <Router>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/popup" element={<PopupWindow />} />
      </Routes>
    </Router>
  </ThemeProvider>
  )
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Main />
  </React.StrictMode>
)