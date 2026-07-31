import { useTranslations } from "use-intl";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ClipboardToggle } from "@/components/ClipboardToggle";
import { Globe, Monitor, Sun, Moon, FolderOpen } from "lucide-react";

interface SettingsPageProps {
  language: string;
  theme: string;
  downloadDir: string;
  concurrent: number;
  onLanguageChange: (lang: string) => void;
  onThemeChange: (theme: string) => void;
  onDownloadDirChange: (dir: string) => void;
  onConcurrentChange: (n: number) => void;
  onClose: () => void;
}

export function SettingsPage({
  language,
  theme,
  downloadDir,
  concurrent,
  onLanguageChange,
  onThemeChange,
  onDownloadDirChange: _onDownloadDirChange,
  onConcurrentChange,
  onClose,
}: SettingsPageProps) {
  const t = useTranslations("settings");

  return (
    <div className="w-full max-w-2xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-semibold">{t("title")}</h2>
        <Button variant="ghost" onClick={onClose}>
          Done
        </Button>
      </div>

      {/* Language */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div className="flex items-center gap-3">
            <Globe className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="font-medium">{t("language")}</p>
            </div>
          </div>
          <Select value={language} onValueChange={onLanguageChange}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zh-TW">繁體中文</SelectItem>
              <SelectItem value="zh-CN">简体中文</SelectItem>
              <SelectItem value="en">English</SelectItem>
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      {/* Theme */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div className="flex items-center gap-3">
            <Monitor className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="font-medium">{t("theme")}</p>
            </div>
          </div>
          <div className="flex gap-1">
            <Button
              variant={theme === "system" ? "default" : "outline"}
              size="sm"
              onClick={() => onThemeChange("system")}
            >
              <Monitor className="h-4 w-4" />
            </Button>
            <Button
              variant={theme === "light" ? "default" : "outline"}
              size="sm"
              onClick={() => onThemeChange("light")}
            >
              <Sun className="h-4 w-4" />
            </Button>
            <Button
              variant={theme === "dark" ? "default" : "outline"}
              size="sm"
              onClick={() => onThemeChange("dark")}
            >
              <Moon className="h-4 w-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Download directory */}
      <Card>
        <CardContent className="flex items-center justify-between py-4 gap-3">
          <div className="flex items-center gap-3 min-w-0">
            <FolderOpen className="h-5 w-5 text-muted-foreground shrink-0" />
            <div className="min-w-0">
              <p className="font-medium">{t("download_dir")}</p>
              <p className="text-sm text-muted-foreground truncate">
                {downloadDir || "~/Downloads"}
              </p>
            </div>
          </div>
          <Button variant="outline" size="sm" className="shrink-0">
            Browse
          </Button>
        </CardContent>
      </Card>

      {/* Concurrent downloads */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div>
            <p className="font-medium">{t("concurrent")}</p>
            <p className="text-sm text-muted-foreground">
              {concurrent} downloads at once
            </p>
          </div>
          <Input
            type="number"
            min={1}
            max={60}
            value={concurrent}
            onChange={(e) => {
              const n = Number.parseInt(e.target.value);
              if (n >= 1 && n <= 60) onConcurrentChange(n);
            }}
            className="w-20 text-center"
          />
        </CardContent>
      </Card>

      {/* Clipboard monitor */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <ClipboardToggle />
        </CardContent>
      </Card>

      {/* yt-dlp version */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div>
            <p className="font-medium">{t("ytdlp_update")}</p>
            <p className="text-sm text-muted-foreground">
              {t("up_to_date")}
            </p>
          </div>
          <Button variant="outline" size="sm">
            {t("ytdlp_update")}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
