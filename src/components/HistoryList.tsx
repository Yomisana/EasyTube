import { useTranslations } from "use-intl";
import { useHistory, type HistoryEntry } from "@/hooks/use-history";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Trash2,
  Download,
  CheckCircle,
  XCircle,
  Ban,
  Clock,
} from "lucide-react";

export function HistoryList() {
  const t = useTranslations("history");
  const { entries, isLoading, removeEntry, clearAll } = useHistory();

  if (isLoading) {
    return (
      <div className="w-full max-w-2xl mx-auto py-8 text-center text-muted-foreground">
        Loading...
      </div>
    );
  }

  if (entries.length === 0) {
    return (
      <div className="w-full max-w-2xl mx-auto py-16 text-center">
        <Clock className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
        <p className="text-lg text-muted-foreground">{t("empty")}</p>
      </div>
    );
  }

  const statusIcon = (status: HistoryEntry["status"]) => {
    switch (status) {
      case "done":
        return <CheckCircle className="h-5 w-5 text-green-500 shrink-0" />;
      case "failed":
        return <XCircle className="h-5 w-5 text-red-500 shrink-0" />;
      case "cancelled":
        return <Ban className="h-5 w-5 text-amber-500 shrink-0" />;
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">{t("title")}</h2>
        {entries.length > 0 && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => clearAll()}
            className="text-muted-foreground"
          >
            <Trash2 className="mr-1 h-4 w-4" />
            {t("clear")}
          </Button>
        )}
      </div>

      <div className="space-y-2">
        {entries.map((entry) => (
          <Card key={entry.jobId}>
            <CardContent className="flex items-center gap-3 py-3 px-4">
              {statusIcon(entry.status)}
              <div className="flex-1 min-w-0">
                <p className="font-medium line-clamp-1">{entry.title || entry.url}</p>
                <p className="text-sm text-muted-foreground">
                  {entry.resolution && `${entry.resolution} · `}
                  {new Date(entry.downloadedAt).toLocaleDateString()}
                </p>
              </div>
              <div className="flex gap-1 shrink-0">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => removeEntry(entry.jobId)}
                  aria-label="Remove"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" aria-label="Redownload">
                  <Download className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
