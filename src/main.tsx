import React from "react";
import ReactDOM from "react-dom/client";
import { IntlProvider } from "use-intl";
import App from "./App";
import { getLocaleFromSystem, getMessages, type Locale } from "./i18n/config";

async function bootstrap() {
  const locale: Locale = getLocaleFromSystem();
  const messages = await getMessages(locale);

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <IntlProvider locale={locale} messages={messages}>
        <App />
      </IntlProvider>
    </React.StrictMode>,
  );
}

void bootstrap();
