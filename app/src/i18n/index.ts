import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import en from "./locales/en.json";
import zh from "./locales/zh.json";
import es from "./locales/es.json";

export const supportedLanguages = [
  { code: "en", label: "English", nativeLabel: "English" },
  { code: "zh", label: "Chinese (Simplified)", nativeLabel: "中文（简体）" },
  { code: "es", label: "Spanish", nativeLabel: "Español" },
] as const;

export type SupportedLanguageCode = (typeof supportedLanguages)[number]["code"];

const resources = {
  en: { translation: en },
  zh: { translation: zh },
  es: { translation: es },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    supportedLngs: ["en", "zh", "es"],
    // Map variants like zh-CN, zh-TW, es-MX, en-US to base languages
    nonExplicitSupportedLngs: true,
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "holdem-language",
      caches: ["localStorage"],
    },
    interpolation: {
      escapeValue: false,
    },
  });

export const setLanguage = (lng: string) => {
  return i18n.changeLanguage(lng);
};

export default i18n;
