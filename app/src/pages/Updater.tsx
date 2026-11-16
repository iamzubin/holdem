import React, { useEffect, useRef, useState } from "react";
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import { closeCurrentWindow } from "@/lib/windowUtils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useTranslation } from "react-i18next";
import { CircleCheck, CircleX, X } from "lucide-react";

interface UpdateInfo {
  version: string;
  notes: string;
}

type UpdateStatus = "idle" | "checking" | "available" | "no_update" | "error" | "downloading" | "downloaded" | "installing";

/** Release notes arrive as plain text, usually "- item" lines: show those
 *  as a list and everything else as paragraphs. */
const ReleaseNotes: React.FC<{ text: string }> = ({ text }) => {
  const lines = text.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  const bullet = /^[-*•]\s+/;
  if (lines.length > 0 && lines.every((line) => bullet.test(line))) {
    return (
      <ul className="list-disc space-y-0.5 ps-4">
        {lines.map((line, i) => (
          <li key={i}>{line.replace(bullet, "")}</li>
        ))}
      </ul>
    );
  }
  return (
    <div className="space-y-1">
      {lines.map((line, i) => (
        <p key={i}>{line.replace(bullet, "• ")}</p>
      ))}
    </div>
  );
};

const Updater: React.FC = () => {
  const { t } = useTranslation();
  // null while the download size is unknown (no Content-Length header).
  const [progress, setProgress] = useState<number | null>(null);
  const [status, setStatus] = useState<UpdateStatus>("idle");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [errorMessage, setErrorMessage] = useState<string>("");
  // Which step failed, so the title is accurate and Retry repeats that step.
  const [failedStep, setFailedStep] = useState<"check" | "download">("check");
  const updateRef = useRef<Update | null>(null);
  const [currentVersion, setCurrentVersion] = useState<string>("");

  useEffect(() => {
    getVersion().then(setCurrentVersion).catch(() => {});
    checkForUpdates();
  }, []);

  const checkForUpdates = async () => {
    try {
      setStatus("checking");

      const update = await check();

      if (update) {
        updateRef.current = update;
        setUpdateInfo({
          version: update.version,
          notes: update.body || t("updater.noNotes")
        });
        // Wait for the user: installing relaunches the app.
        setStatus("available");
      } else {
        setStatus("no_update");
      }
    } catch (error) {
      setFailedStep("check");
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : String(error));
      // The raw message stays available under "Details".
    }
  };

  const downloadAndInstallUpdate = async () => {
    const update = updateRef.current;
    if (!update) return;
    try {
      setStatus("downloading");
      setProgress(null);
      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setProgress(Math.min(Math.round((downloaded / contentLength) * 100), 100));
            }
            break;
          case 'Finished':
            setStatus("downloaded");
            break;
        }
      });

      setStatus("installing");

      // Short delay before relaunch
      setTimeout(async () => {
        await relaunch();
      }, 1000);
    } catch (error) {
      setFailedStep("download");
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : String(error));
    }
  };

  const renderContent = () => {
    switch (status) {
      case "checking":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-4 border-t-primary border-muted mb-4" />
              <span className="text-muted-foreground">{t("updater.checking")}</span>
            </CardContent>
          </Card>
        );
      case "downloading":
        return (
          <Card>
            <CardContent className="py-8">
              <Progress
                value={progress ?? 0}
                className="mb-4"
                aria-label={t("updater.downloading")}
              />
              {progress !== null && <div className="text-center text-sm mb-2">{progress}%</div>}
              <span className="text-muted-foreground">{t("updater.downloading")}</span>
            </CardContent>
          </Card>
        );
      case "downloaded":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <CircleCheck className="mb-3 h-10 w-10 text-primary" aria-hidden />
              <span className="font-medium">{t("updater.downloadedTitle")}</span>
              <span className="text-muted-foreground mt-2">{t("updater.downloadedDesc")}</span>
            </CardContent>
          </Card>
        );
      case "installing":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-4 border-t-primary border-muted mb-4" />
              <span className="text-muted-foreground">{t("updater.installing")}</span>
            </CardContent>
          </Card>
        );
      case "available":
        return (
          <Card>
            <CardHeader>
              <CardTitle>{t("updater.availableTitle")}</CardTitle>
            </CardHeader>
            <CardContent>
              {updateInfo && (
                <>
                  <div className="text-primary font-semibold mb-2">
                    {t("updater.version", { version: updateInfo.version })}
                  </div>
                  <Alert className="mb-4">
                    <AlertTitle>{t("updater.releaseNotes")}</AlertTitle>
                    <AlertDescription className="max-h-28 overflow-y-auto">
                      <ReleaseNotes text={updateInfo.notes} />
                    </AlertDescription>
                  </Alert>
                  <p className="text-sm text-muted-foreground mb-4">{t("updater.restartWarning")}</p>
                  <div className="flex gap-2">
                    <Button className="flex-1" onClick={downloadAndInstallUpdate}>
                      {t("updater.installNow")}
                    </Button>
                    <Button className="flex-1" variant="outline" onClick={closeCurrentWindow}>
                      {t("updater.later")}
                    </Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        );
      case "no_update":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <CircleCheck className="mb-3 h-10 w-10 text-primary" aria-hidden />
              <span className="font-medium">
                {currentVersion ? t("updater.latestVersion", { version: currentVersion }) : t("updater.latest")}
              </span>
              <Button className="mt-4" variant="outline" onClick={closeCurrentWindow}>
                {t("updater.close")}
              </Button>
            </CardContent>
          </Card>
        );
      case "error":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <CircleX className="mb-3 h-10 w-10 text-destructive" aria-hidden />
              <span className="font-medium">
                {t(failedStep === "download" ? "updater.downloadErrorTitle" : "updater.errorTitle")}
              </span>
              <span className="mt-2 text-center text-sm text-muted-foreground">
                {t(failedStep === "download" ? "updater.downloadErrorHelp" : "updater.errorHelp")}
              </span>
              <details className="mt-2 max-w-full text-xs text-muted-foreground">
                <summary className="cursor-pointer text-center">{t("common.details")}</summary>
                <p className="mt-1 break-words">{errorMessage}</p>
              </details>
              <Button
                className="mt-4"
                variant="outline"
                onClick={failedStep === "download" ? downloadAndInstallUpdate : checkForUpdates}
              >
                {t("updater.retry")}
              </Button>
            </CardContent>
          </Card>
        );
      default:
        return (
          <Card>
            <CardContent className="py-8">
              <span className="text-muted-foreground">{t("updater.waiting")}</span>
            </CardContent>
          </Card>
        );
    }
  };

  return (

    <div className="relative flex h-full w-full flex-col overflow-y-auto bg-background text-foreground" data-tauri-drag-region>
      <Button
        variant="ghost"
        size="icon"
        className="absolute top-2 end-2"
        onClick={closeCurrentWindow}
        aria-label={t("updater.close")}
      >
        <X className="h-5 w-5" />
      </Button>
      <header className="px-6 pt-4 pb-2" data-tauri-drag-region>
        <h1 data-tauri-drag-region className="text-center text-base font-semibold">{t("updater.title")}</h1>
      </header>
      <div className="px-4 pb-3" aria-live="polite">{renderContent()}</div>
    </div>
  );
};

export default Updater;
