import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { AlertTriangle } from 'lucide-react';

interface DuplicateDialogProps {
  open: boolean;
  title: string;
  resolution?: string;
  downloadedAt: string;
  onRedownload: () => void;
  onCancel: () => void;
}

export function DuplicateDialog({
  open,
  title,
  resolution,
  downloadedAt,
  onRedownload,
  onCancel,
}: DuplicateDialogProps) {
  const dateStr = new Date(downloadedAt).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onCancel()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <AlertTriangle className="h-6 w-6 text-amber-500" />
            <DialogTitle className="text-xl">Already downloaded</DialogTitle>
          </div>
          <DialogDescription asChild>
            <div className="pt-2 space-y-2 text-base">
              <p className="font-medium">{title}</p>
              {resolution && (
                <p>
                  Resolution: <span className="font-medium">{resolution}</span>
                </p>
              )}
              <p className="text-muted-foreground">Downloaded: {dateStr}</p>
              <p className="pt-2">
                This video was already downloaded with the same quality
                settings. Are you sure you want to download it again?
              </p>
            </div>
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="gap-2 sm:gap-0">
          <Button variant="outline" onClick={onCancel} className="h-12">
            Cancel
          </Button>
          <Button onClick={onRedownload} className="h-12 flex-1">
            Download Again
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
