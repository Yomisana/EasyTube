import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Download, X } from 'lucide-react';
import { useTranslations } from 'use-intl';

interface DetectedDialogProps {
  open: boolean;
  url: string;
  title?: string;
  onDownload: (url: string) => void;
  onDismiss: () => void;
  onBlockDomain?: (url: string) => void;
}

export function DetectedDialog({
  open,
  url,
  title,
  onDownload,
  onDismiss,
  onBlockDomain,
}: DetectedDialogProps) {
  const t = useTranslations('clipboard');

  const displayUrl = url.replace(/^https?:\/\//, '').replace(/\/$/, '');
  const truncatedUrl =
    displayUrl.length > 50 ? `${displayUrl.slice(0, 47)}...` : displayUrl;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onDismiss()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="text-xl">{t('detected')}</DialogTitle>
          <DialogDescription className="text-base">
            {title ?? truncatedUrl}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={() => onBlockDomain?.(url)}
            className="h-12"
          >
            <X className="mr-2 h-5 w-5" />
            {t('no')}
          </Button>
          <Button onClick={() => onDownload(url)} className="h-12 flex-1">
            <Download className="mr-2 h-5 w-5" />
            {t('yes')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
