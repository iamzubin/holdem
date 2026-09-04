import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

// Auto-load every locale file dropped into ./locales (e.g. by translators
// or a Crowdin sync). Filename (without .json) is the language code.
const localeModules = import.meta.glob("./locales/*.json", {
  eager: true,
}) as Record<string, { default: Record<string, unknown> }>;

const resources: Record<string, { translation: Record<string, unknown> }> = {};
for (const [path, mod] of Object.entries(localeModules)) {
  const code = path.split("/").pop()!.replace(/\.json$/, "");
  resources[code] = { translation: mod.default };
}

// Backwards-compat aliases for preferences stored before regional codes
// existed (Settings used to offer short codes `zh` / `es`).
if (resources["zh-CN"]) resources["zh"] = resources["zh-CN"];
if (resources["es-ES"]) resources["es"] = resources["es-ES"];

export const supportedLanguages = [
  { code: "en", label: "English", nativeLabel: "English" },
  { code: "ar", label: "Arabic", nativeLabel: "العربية" },
  { code: "cs", label: "Czech", nativeLabel: "Čeština" },
  { code: "da", label: "Danish", nativeLabel: "Dansk" },
  { code: "de", label: "German", nativeLabel: "Deutsch" },
  { code: "el", label: "Greek", nativeLabel: "Ελληνικά" },
  { code: "es-ES", label: "Spanish (Spain)", nativeLabel: "Español (España)" },
  { code: "fi", label: "Finnish", nativeLabel: "Suomi" },
  { code: "fr", label: "French", nativeLabel: "Français" },
  { code: "it", label: "Italian", nativeLabel: "Italiano" },
  { code: "ja", label: "Japanese", nativeLabel: "日本語" },
  { code: "ko", label: "Korean", nativeLabel: "한국어" },
  { code: "nl", label: "Dutch", nativeLabel: "Nederlands" },
  { code: "no", label: "Norwegian", nativeLabel: "Norsk" },
  { code: "pl", label: "Polish", nativeLabel: "Polski" },
  { code: "pt-BR", label: "Portuguese (Brazil)", nativeLabel: "Português (Brasil)" },
  { code: "pt-PT", label: "Portuguese (Portugal)", nativeLabel: "Português (Portugal)" },
  { code: "ro", label: "Romanian", nativeLabel: "Română" },
  { code: "ru", label: "Russian", nativeLabel: "Русский" },
  { code: "sv-SE", label: "Swedish", nativeLabel: "Svenska" },
  { code: "tr", label: "Turkish", nativeLabel: "Türkçe" },
  { code: "uk", label: "Ukrainian", nativeLabel: "Українська" },
  { code: "vi", label: "Vietnamese", nativeLabel: "Tiếng Việt" },
  { code: "zh-CN", label: "Chinese (Simplified)", nativeLabel: "中文（简体）" },
  { code: "zh-TW", label: "Chinese (Traditional)", nativeLabel: "中文（繁體）" },
] as const;

export type SupportedLanguageCode = (typeof supportedLanguages)[number]["code"];

// Keep <html lang>/<html dir> in sync so RTL languages (ar, he)
// lay out correctly.
const syncDocumentAttrs = (lng: string) => {
  if (typeof document !== "undefined") {
    document.documentElement.lang = lng;
    document.documentElement.dir = i18n.dir(lng);
  }
};

i18n.on("languageChanged", syncDocumentAttrs);

export const i18nInitialization = i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    supportedLngs: Object.keys(resources),
    // NOTE: do NOT enable nonExplicitSupportedLngs. In i18next >= 23 it makes
    // isSupportedCode() reduce every regional code to its base language
    // first, which silently rejects explicitly listed codes whose base is
    // not listed (pt-BR, pt-PT, sv-SE all fell back to English).
    // Variants like en-US or pt-MZ are still mapped to the closest match
    // (en, pt-BR) by the detector's best-match lookup.
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "holdem-language",
      caches: ["localStorage"],
    },
    interpolation: {
      escapeValue: false,
    },
  })
  .then(() => {
    syncDocumentAttrs(i18n.language);
  });

export const setLanguage = (lng: string) => {
  return i18n.changeLanguage(lng);
};

export default i18n;
