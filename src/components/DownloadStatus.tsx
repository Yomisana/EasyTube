import { useTranslations } from "use-intl";
import { Progress } from "@/components/ui/progress";
import { Card, CardContent } from "@/components/ui/card";
import { CheckCircle, XCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

interface DownloadStatusProps {
  stage: "probing" | "downloading" | "done" | "error";
  title?: string;
  percent?: number;
  error?: string;
  onRetry?: () => void;
  onOpenFolder?: () => void;
  onReset?: () => void;
}

export function DownloadStatus({
  stage,
  title,
  percent = 0,
  error,
  onRetry,
  onOpenFolder,
  onReset,
}: DownloadStatusProps) {
  const t = useTranslations("progress");

  if (stage === "probing") {
    return (
      <Card className="w-full max-w-2xl">
        <CardContent className="flex flex-col items-center gap-4 py-12">
          <Loader2 className="h-10 w-10 animate-spin text-muted-foreground" />
          <p className="text-lg font-medium">{t("preparing")}</p>
          <Progress value={undefined} className="w-64 h-1" />
        </CardContent>
      </Card>
    );
  }

  if (stage === "downloading") {
    return (
      <Card className="w-full max-w-2xl">
        <CardContent className="flex flex-col items-center gap-4 py-8">
          {title && (
            <p className="text-lg font-medium line-clamp-1 text-center">
              {title}
            </p>
          )}
          <div className="w-full space-y-2">
            <Progress value={percent} className="w-full h-3" />
            <p className="text-center text-2xl font-bold">
              {Math.round(percent)}%
            </p>
          </div>
          <p className="text-muted-foreground">{t("downloading")}</p>
        </CardContent>
      </Card>
    );
  }

  if (stage === "done") {
    return (
      <Card className="w-full max-w-2xl border-green-200 dark:border-green-800">
        <CardContent className="flex flex-col items-center gap-4 py-10">
          <CheckCircle className="h-14 w-14 text-green-500" />
          <p className="text-2xl font-semibold">{t("done")}</p>
          {title && (
            <p className="text-muted-foreground line-clamp-2 text-center">
              {title}
            </p>
          )}
          <div className="flex gap-3 pt-2">
            {onOpenFolder && (
              <Button
                variant="outline"
                size="lg"
                onClick={onOpenFolder}
                className="h-12"
              >
                {t("open_folder")}
              </Button>
            )}
            {onReset && (
              <Button size="lg" onClick={onReset} className="h-12">
                Download Another
              </Button>
            )}
          </div>
        </CardContent>
      </Card>
    );
  }

  // error
  return (
    <Card className="w-full max-w-2xl border-red-200 dark:border-red-800">
      <CardContent className="flex flex-col items-center gap-4 py-10">
        <XCircle className="h-14 w-14 text-red-500" />
        <p className="text-2xl font-semibold">{t("failed")}</p>
        {error && (
          <p className="text-muted-foreground text-center max-w-md">
            {error}
          </p>
        )}
        <div className="flex gap-3 pt-2">
          {onRetry && (
            <Button onClick={onRetry} size="lg" className="h-12">
              {t("retry")}
            </Button>
          )}
          {onReset && (
            <Button
              variant="outline"
              size="lg"
              onClick={onReset}
              className="h-12"
            >
              Try Another URL
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
