import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { ChevronDown, Download, Film, Music } from 'lucide-react';
import { useTranslations } from 'use-intl';

interface FormatInfo {
  id: string;
  ext: string;
  resolution?: string;
  note?: string;
}

interface ProbeResultProps {
  title: string;
  thumbnail?: string;
  duration?: string;
  formats: FormatInfo[];
  selectedFormat: string;
  onSelectFormat: (id: string) => void;
  onDownload: () => void;
  onCancel: () => void;
}

export function ProbeResult({
  title,
  thumbnail,
  duration,
  formats,
  selectedFormat,
  onSelectFormat,
  onDownload,
  onCancel,
}: ProbeResultProps) {
  const t = useTranslations('hero');

  const bestFormats = formats.filter(
    (f) =>
      f.resolution &&
      ['2160', '1440', '1080', '720', '480'].includes(f.resolution) &&
      f.ext === 'mp4',
  );

  const audioFormats = formats.filter(
    (f) => f.ext === 'm4a' || f.ext === 'mp3',
  );
  const hasMore = formats.length > bestFormats.length + audioFormats.length;

  return (
    <Card className="w-full max-w-2xl animate-in fade-in slide-in-from-bottom-4 duration-300">
      <CardContent className="flex flex-col gap-4 pt-6">
        <div className="flex gap-4">
          {thumbnail && (
            <img
              src={thumbnail}
              alt={title}
              className="w-40 h-24 rounded-lg object-cover shrink-0"
            />
          )}
          <div className="flex-1 min-w-0">
            <h2 className="text-lg font-semibold leading-tight line-clamp-2">
              {title}
            </h2>
            {duration && (
              <p className="text-sm text-muted-foreground mt-1">{duration}</p>
            )}
          </div>
        </div>

        <div className="space-y-2">
          <p className="text-sm font-medium">{t('quality')}</p>
          <div className="flex flex-wrap gap-2">
            <Button
              variant={selectedFormat === 'best' ? 'default' : 'outline'}
              size="sm"
              onClick={() => onSelectFormat('best')}
            >
              {t('best')}
            </Button>
            {bestFormats.map((f) => (
              <Button
                key={f.id}
                variant={selectedFormat === f.id ? 'default' : 'outline'}
                size="sm"
                onClick={() => onSelectFormat(f.id)}
              >
                <Film className="mr-1 h-4 w-4" />
                {f.resolution}p
              </Button>
            ))}
            {audioFormats.length > 0 && (
              <Button
                variant={
                  selectedFormat === audioFormats[0].id ? 'default' : 'outline'
                }
                size="sm"
                onClick={() => onSelectFormat(audioFormats[0].id)}
              >
                <Music className="mr-1 h-4 w-4" />
                Audio
              </Button>
            )}
            {hasMore && (
              <Button variant="outline" size="sm" disabled>
                <ChevronDown className="mr-1 h-4 w-4" />
                More
              </Button>
            )}
          </div>
        </div>

        <div className="flex gap-3 pt-2">
          <Button variant="outline" onClick={onCancel} className="h-12">
            Cancel
          </Button>
          <Button onClick={onDownload} className="h-12 flex-1 text-lg">
            <Download className="mr-2 h-5 w-5" />
            {t('download')}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
