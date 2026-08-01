import { ClipboardToggle } from '@/components/ClipboardToggle';
import { DetectedDialog } from '@/components/DetectedDialog';
import { DownloadStatus } from '@/components/DownloadStatus';
import { DuplicateDialog } from '@/components/DuplicateDialog';
import { HistoryList } from '@/components/HistoryList';
import { ProbeResult } from '@/components/ProbeResult';
import { SettingsPage } from '@/components/SettingsPage';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { addHistoryEntry, checkDuplicateDownload } from '@/lib/history';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openPath } from '@tauri-apps/plugin-opener';
import { Download, History, Settings } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useLocale, useTranslations } from 'use-intl';

type Tab = 'home' | 'history' | 'settings';

interface FormatInfo {
  id: string;
  ext: string;
  resolution?: string;
  note?: string;
}

interface VideoInfo {
  title: string;
  url: string;
  thumbnail?: string;
  formats: FormatInfo[];
  duration?: string;
}

interface DownloadProgress {
  job_id: string;
  stage: string;
  percent: number;
}

interface DownloadComplete {
  job_id: string;
  title?: string;
  output_file?: string;
}

interface DownloadFailed {
  job_id: string;
  error: string;
}

function App() {
  const t = useTranslations('hero');
  const locale = useLocale();
  const [tab, setTab] = useState<Tab>('home');

  // Download state machine
  const [stage, setStage] = useState<
    'idle' | 'probing' | 'probed' | 'downloading' | 'done' | 'error'
  >('idle');
  const [url, setUrl] = useState('');
  const [videoInfo, setVideoInfo] = useState<VideoInfo | null>(null);
  const [selectedFormat, setSelectedFormat] = useState('best');
  const [downloadPercent, setDownloadPercent] = useState(0);
  const [currentJobId, setCurrentJobId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState('');
  const [outputFile, setOutputFile] = useState<string | undefined>();

  // Duplicate dialog
  const [dupOpen, setDupOpen] = useState(false);
  const [dupInfo, setDupInfo] = useState<{
    title: string;
    resolution?: string;
    downloadedAt: string;
  } | null>(null);
  const [pendingDupDownload, setPendingDupDownload] = useState<
    (() => void) | null
  >(null);

  // Clipboard detection
  const [detectedUrl, setDetectedUrl] = useState<string | null>(null);

  // Settings
  const [language, setLanguage] = useState<string>(locale);
  const [theme, setTheme] = useState('system');
  const [downloadDir, setDownloadDir] = useState('');
  const [concurrent, setConcurrent] = useState(5);

  // Listen for download events
  useEffect(() => {
    const unlisten1 = listen<DownloadProgress>('download-progress', (event) => {
      setDownloadPercent(event.payload.percent);
      if (
        event.payload.stage === 'downloading' ||
        event.payload.stage === 'probing'
      ) {
        setStage('downloading');
      }
    });
    const unlisten2 = listen<DownloadComplete>('download-complete', (event) => {
      setStage('done');
      setOutputFile(event.payload.output_file);
      if (event.payload.title && url) {
        addHistoryEntry({
          jobId: event.payload.job_id,
          url,
          title: event.payload.title,
          formatId: selectedFormat,
          resolution: videoInfo?.formats.find((f) => f.id === selectedFormat)
            ?.resolution,
          outputFile: event.payload.output_file,
          status: 'done',
          downloadedAt: new Date().toISOString(),
        }).catch(console.error);
      }
    });
    const unlisten3 = listen<DownloadFailed>('download-failed', (event) => {
      setStage('error');
      setErrorMsg(event.payload.error);
    });
    return () => {
      unlisten1.then((u) => u());
      unlisten2.then((u) => u());
      unlisten3.then((u) => u());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url, selectedFormat, videoInfo]);

  const handleProbe = useCallback(async (inputUrl: string) => {
    const cleanUrl = inputUrl.trim();
    if (!cleanUrl) return;
    setUrl(cleanUrl);
    setStage('probing');
    setDownloadPercent(0);
    setErrorMsg('');
    try {
      const info: VideoInfo = await invoke('probe_url', { url: cleanUrl });
      setVideoInfo(info);
      setStage('probed');
    } catch (e) {
      setStage('error');
      setErrorMsg(String(e));
    }
  }, []);

  const handleDownload = useCallback(async () => {
    if (!url || !videoInfo) return;

    const formatId = selectedFormat;
    const existing = await checkDuplicateDownload(url, formatId);
    if (existing) {
      setDupInfo({
        title: existing.title,
        resolution: existing.resolution,
        downloadedAt: existing.downloadedAt,
      });
      setPendingDupDownload(() => async () => {
        setDupOpen(false);
        await doDownload();
      });
      setDupOpen(true);
      return;
    }

    await doDownload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url, videoInfo, selectedFormat]);

  async function doDownload() {
    setStage('downloading');
    setDownloadPercent(0);
    try {
      const jobId: string = await invoke('start_download', {
        url,
        formatId: selectedFormat === 'best' ? null : selectedFormat,
      });
      setCurrentJobId(jobId);
    } catch (e) {
      setStage('error');
      setErrorMsg(String(e));
    }
  }

  const handleReset = useCallback(() => {
    setStage('idle');
    setUrl('');
    setVideoInfo(null);
    setSelectedFormat('best');
    setDownloadPercent(0);
    setCurrentJobId(null);
    setErrorMsg('');
    setOutputFile(undefined);
  }, []);

  const handleCancelDownload = useCallback(async () => {
    if (currentJobId) {
      try {
        await invoke('cancel_download', { jobId: currentJobId });
      } catch (e) {
        console.error(e);
      }
    }
    handleReset();
  }, [currentJobId, handleReset]);

  const handleOpenFolder = useCallback(() => {
    if (outputFile) {
      const dir = outputFile.split('/').slice(0, -1).join('/') || '/';
      openPath(dir).catch(() => {});
    }
  }, [outputFile]);

  // Keyboard shortcut: Escape to cancel
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && stage === 'downloading') {
        handleCancelDownload();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [stage, handleCancelDownload]);

  return (
    <div className="flex flex-col min-h-screen bg-background">
      <header className="flex items-center justify-between px-6 py-4 border-b">
        <h1 className="text-2xl font-bold">EasyTube</h1>
        <nav className="flex gap-1" role="tablist">
          <Button
            variant={tab === 'home' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setTab('home')}
            role="tab"
            aria-selected={tab === 'home'}
          >
            <Download className="mr-1 h-4 w-4" />
            Home
          </Button>
          <Button
            variant={tab === 'history' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setTab('history')}
            role="tab"
            aria-selected={tab === 'history'}
          >
            <History className="mr-1 h-4 w-4" />
            History
          </Button>
          <Button
            variant={tab === 'settings' ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setTab('settings')}
            role="tab"
            aria-selected={tab === 'settings'}
          >
            <Settings className="mr-1 h-4 w-4" />
            Settings
          </Button>
        </nav>
      </header>

      <main className="flex-1 flex flex-col items-center px-4 py-8">
        {tab === 'history' && <HistoryList />}
        {tab === 'settings' && (
          <SettingsPage
            language={language}
            theme={theme}
            downloadDir={downloadDir}
            concurrent={concurrent}
            onLanguageChange={setLanguage}
            onThemeChange={setTheme}
            onDownloadDirChange={setDownloadDir}
            onConcurrentChange={setConcurrent}
            onClose={() => setTab('home')}
          />
        )}

        {tab === 'home' && (
          <>
            {stage === 'idle' && (
              <Card className="w-full max-w-2xl">
                <CardContent className="flex flex-col gap-6 pt-8">
                  <h2 className="text-3xl font-semibold text-center leading-tight">
                    {t('description')}
                  </h2>
                  <div className="flex gap-3">
                    <Input
                      className="text-lg h-14 px-4"
                      placeholder={t('placeholder')}
                      value={url}
                      onChange={(e) => setUrl(e.target.value)}
                      onPaste={(e) => {
                        setTimeout(() => {
                          const pasted = e.currentTarget.value.trim();
                          if (pasted) handleProbe(pasted);
                        }, 100);
                      }}
                      onKeyDown={(e) => e.key === 'Enter' && handleProbe(url)}
                      aria-label={t('placeholder')}
                      autoFocus
                    />
                    <Button
                      className="h-14 px-8 text-lg"
                      size="lg"
                      onClick={() => handleProbe(url)}
                      disabled={!url.trim()}
                    >
                      <Download className="mr-2 h-5 w-5" />
                      {t('download')}
                    </Button>
                  </div>
                  <div className="flex justify-center">
                    <ClipboardToggle
                      onToggle={async (enabled) => {
                        await invoke('set_clipboard_enabled', {
                          enabled,
                        }).catch(() => {});
                      }}
                    />
                  </div>
                </CardContent>
              </Card>
            )}

            {stage === 'probing' && <DownloadStatus stage="probing" />}

            {stage === 'probed' && videoInfo && (
              <ProbeResult
                title={videoInfo.title}
                thumbnail={videoInfo.thumbnail}
                duration={videoInfo.duration}
                formats={videoInfo.formats}
                selectedFormat={selectedFormat}
                onSelectFormat={setSelectedFormat}
                onDownload={handleDownload}
                onCancel={handleReset}
              />
            )}

            {stage === 'downloading' && (
              <DownloadStatus
                stage="downloading"
                title={videoInfo?.title}
                percent={downloadPercent}
                onOpenFolder={handleOpenFolder}
                onReset={handleCancelDownload}
              />
            )}

            {stage === 'done' && (
              <DownloadStatus
                stage="done"
                title={videoInfo?.title}
                onOpenFolder={handleOpenFolder}
                onReset={handleReset}
              />
            )}

            {stage === 'error' && (
              <DownloadStatus
                stage="error"
                error={errorMsg}
                onRetry={videoInfo ? handleDownload : () => handleProbe(url)}
                onReset={handleReset}
              />
            )}
          </>
        )}
      </main>

      <DuplicateDialog
        open={dupOpen}
        title={dupInfo?.title ?? ''}
        resolution={dupInfo?.resolution}
        downloadedAt={dupInfo?.downloadedAt ?? ''}
        onRedownload={() => {
          setDupOpen(false);
          pendingDupDownload?.();
        }}
        onCancel={() => {
          setDupOpen(false);
          setPendingDupDownload(null);
        }}
      />

      {detectedUrl && (
        <DetectedDialog
          open={!!detectedUrl}
          url={detectedUrl}
          onDownload={(u) => {
            setDetectedUrl(null);
            handleProbe(u);
          }}
          onDismiss={() => setDetectedUrl(null)}
        />
      )}
    </div>
  );
}

export default App;
