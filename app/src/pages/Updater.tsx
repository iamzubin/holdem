import React, { useEffect, useState } from "react";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { useTranslation } from "react-i18next";

interface UpdateInfo {
  version: string;
  notes: string;
}

type UpdateStatus = "idle" | "checking" | "available" | "no_update" | "error" | "downloading" | "downloaded" | "installing";

const Updater: React.FC = () => {
  const { t } = useTranslation();
  const [progress, setProgress] = useState<number>(0);
  const [status, setStatus] = useState<UpdateStatus>("idle");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [errorMessage, setErrorMessage] = useState<string>("");

  useEffect(() => {
    checkForUpdates();
  }, []);

  const checkForUpdates = async () => {
    try {
      setStatus("checking");

      const update = await check();

      if (update) {
        setUpdateInfo({
          version: update.version,
          notes: update.body || t("updater.noNotes")
        });
        setStatus("available");

        // Start download after a short delay
        setTimeout(() => {
          downloadAndInstallUpdate(update);
        }, 3000);
      } else {
        setStatus("no_update");
      }
    } catch (error) {
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : String(error));
    }
  };

  const downloadAndInstallUpdate = async (update: any) => {
    try {
      setStatus("downloading");
      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const percentage = Math.min(Math.round((downloaded / contentLength) * 100), 100);
            setProgress(percentage);
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
      setStatus("error");
      setErrorMessage(`Failed to download and install update: ${error instanceof Error ? error.message : String(error)}`);
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
              <Progress value={progress} className="mb-4" />
              <div className="text-center text-sm mb-2">{progress}%</div>
              <span className="text-muted-foreground">{t("updater.downloading")}</span>
            </CardContent>
          </Card>
        );
      case "downloaded":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <div className="text-green-600 text-4xl mb-3">✓</div>
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
                    <AlertDescription className="whitespace-pre-line">
                      {updateInfo.notes}
                    </AlertDescription>
                  </Alert>
                  <span className="text-muted-foreground">{t("updater.autoDownload")}</span>
                </>
              )}
            </CardContent>
          </Card>
        );
      case "no_update":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <div className="text-green-600 text-4xl mb-3">✓</div>
              <span className="font-medium">{t("updater.latest")}</span>
            </CardContent>
          </Card>
        );
      case "error":
        return (
          <Card>
            <CardContent className="flex flex-col items-center py-8">
              <div className="text-destructive text-4xl mb-3">✗</div>
              <span className="font-medium">{t("updater.errorTitle")}</span>
              <span className="text-destructive mt-2">{errorMessage}</span>
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

    <Card className="relative w-full h-full" data-tauri-drag-region>
      <Button
        variant="ghost"
        size="icon"
        className="absolute top-2 right-2"
        onClick={() => getCurrentWindow().close()}
        aria-label={t("updater.close")}
      >
        <svg className="w-6 h-6" fill="none" stroke="#ff0000" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </Button>
      <CardHeader data-tauri-drag-region>
        <CardTitle data-tauri-drag-region className="text-center">{t("updater.title")}</CardTitle>
      </CardHeader>
      <CardContent>{renderContent()}</CardContent>
    </Card>
  );
};

export default Updater;
