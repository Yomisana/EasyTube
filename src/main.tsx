import React from "react";
import ReactDOM from "react-dom/client";
import { IntlProvider } from "use-intl";
import { PersistQueryClientProvider } from "@tanstack/react-query-persist-client";
import App from "./App";
import { getLocaleFromSystem, getMessages, type Locale } from "./i18n/config";
import { queryClient, persister } from "./lib/query-client";

async function bootstrap() {
  const locale: Locale = getLocaleFromSystem();
  const messages = await getMessages(locale);

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <PersistQueryClientProvider
        client={queryClient}
        persistOptions={{ persister, maxAge: Infinity }}
      >
        <IntlProvider locale={locale} messages={messages}>
          <App />
        </IntlProvider>
      </PersistQueryClientProvider>
    </React.StrictMode>,
  );
}

void bootstrap();
