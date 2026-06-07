import { createApp } from "vue";
import { createI18n } from "vue-i18n";
import "./assets/index.css";
import App from "./App.vue";

import en from "./locales/en.json";
import zh from "./locales/zh.json";

const savedLocale = localStorage.getItem("micyou_language") || "system";

const getSystemLocale = () => {
  return navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
};

let initialLocale = "en";
if (savedLocale === "en" || savedLocale === "English") {
  initialLocale = "en";
} else if (savedLocale === "zh" || savedLocale === "简体中文") {
  initialLocale = "zh";
} else {
  initialLocale = getSystemLocale();
}

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: "en",
  messages: {
    en,
    zh
  }
});

const app = createApp(App);
app.use(i18n);
app.mount("#app");
