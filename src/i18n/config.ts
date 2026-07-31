export type Locale = "zh-TW" | "zh-CN" | "en";

export const locales: Locale[] = ["zh-TW", "zh-CN", "en"];

export const defaultLocale: Locale = "zh-TW";

export function getLocaleFromSystem(): Locale {
  const lang = navigator.language;
  if (lang.startsWith("zh")) {
    if (lang.includes("TW") || lang.includes("HK") || lang.includes("Hant")) {
      return "zh-TW";
    }
    return "zh-CN";
  }
  return "en";
}

export async function getMessages(locale: Locale) {
  return (await import(`./messages/${locale}.json`)).default;
}
