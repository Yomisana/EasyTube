import { useState } from "react";
import { useTranslations } from "use-intl";
import { invoke } from "@tauri-apps/api/core";
import { Download, Settings, History } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

function App() {
  const t = useTranslations("hero");
  const tProgress = useTranslations("progress");
  const [url, setUrl] = useState("");
  const [isProbing, setIsProbing] = useState(false);

  const handleDownload = async () => {
    if (!url.trim()) return;
    setIsProbing(true);
    try {
      await invoke("probe_url", { url: url.trim() });
      await invoke("start_download", { url: url.trim(), formatId: null });
    } catch (e) {
      console.error(e);
    } finally {
      setIsProbing(false);
    }
  };

  return (
    <div className="flex flex-col min-h-screen bg-background">
      <header className="flex items-center justify-between px-6 py-4 border-b">
        <h1 className="text-2xl font-bold">EasyTube</h1>
        <nav className="flex gap-2">
          <Button variant="ghost" size="icon" aria-label="History">
            <History className="h-5 w-5" />
          </Button>
          <Button variant="ghost" size="icon" aria-label="Settings">
            <Settings className="h-5 w-5" />
          </Button>
        </nav>
      </header>

      <main className="flex-1 flex flex-col items-center justify-center px-4 py-16">
        <Card className="w-full max-w-2xl">
          <CardContent className="flex flex-col gap-6 pt-8">
            <h2 className="text-3xl font-semibold text-center leading-tight">
              {t("description")}
            </h2>

            <div className="flex gap-3">
              <Input
                className="text-lg h-14 px-4"
                placeholder={t("placeholder")}
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleDownload()}
              />
              <Button
                className="h-14 px-8 text-lg"
                size="lg"
                onClick={handleDownload}
                disabled={isProbing || !url.trim()}
              >
                <Download className="mr-2 h-5 w-5" />
                {t("download")}
              </Button>
            </div>

            {isProbing && (
              <div className="space-y-2">
                <p className="text-sm text-muted-foreground">{t("probe_hint")}</p>
                <Progress value={undefined} className="h-1" />
              </div>
            )}
          </CardContent>
        </Card>

        <p className="mt-8 text-sm text-muted-foreground">
          {tProgress("open_folder")}
        </p>
      </main>
    </div>
  );
}

export default App;
