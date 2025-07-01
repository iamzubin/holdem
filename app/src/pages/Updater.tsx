import React, { useEffect, useState } from "react";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getCurrentWindow } from "@tauri-apps/api/window";

interface UpdateInfo {
  version: string;
  notes: string;
}

type UpdateStatus = "idle" | "checking" | "available" | "no_update" | "error" | "downloading" | "downloaded" | "installing";

const Updater: React.FC = () => {
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
          notes: update.body || "No release notes available"
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
            console.log(`Started downloading ${contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const percentage = Math.min(Math.round((downloaded / contentLength) * 100), 100);
            setProgress(percentage);
            console.log(`Downloaded ${downloaded} of ${contentLength} bytes (${percentage}%)`);
            break;
          case 'Finished':
            console.log('Download finished');
            setStatus("downloaded");
            break;
        }
      });
      
      console.log('Update installed');
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
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="w-8 h-8 border-4 border-t-blue-500 border-gray-200 rounded-full mx-auto mb-4 animate-spin"></div>
            <p className="text-gray-600">Checking for updates...</p>
          </div>
        );
      case "downloading":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="my-4">
              <div className="w-full bg-gray-200 rounded-full h-3">
                <div 
                  className="bg-blue-500 h-3 rounded-full transition-all duration-300" 
                  style={{ width: `${progress}%` }}
                ></div>
              </div>
              <p className="text-center mt-2">{progress}%</p>
            </div>
            <p className="text-gray-600">Downloading update...</p>
          </div>
        );
      case "downloaded":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="text-green-500 text-4xl mb-3">✓</div>
            <p className="text-gray-700">Download completed!</p>
            <p className="text-gray-600 mt-2">Preparing to install...</p>
          </div>
        );
      case "installing":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="w-8 h-8 border-4 border-t-blue-500 border-gray-200 rounded-full mx-auto mb-4 animate-spin"></div>
            <p className="text-gray-600">Installing update and restarting application...</p>
          </div>
        );
      case "available":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <h3 className="text-xl font-bold text-gray-800 mb-2">Update Available!</h3>
            {updateInfo && (
              <>
                <p className="text-lg font-semibold text-blue-600 mb-4">Version {updateInfo.version}</p>
                <div className="bg-white p-4 rounded-lg border border-gray-200 mb-4 max-h-48 overflow-y-auto text-left">
                  <h4 className="font-medium mb-2">Release Notes:</h4>
                  <p className="text-gray-700 whitespace-pre-line">{updateInfo.notes}</p>
                </div>
                <p className="text-gray-600">Download will start automatically...</p>
              </>
            )}
          </div>
        );
      case "no_update":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="text-green-500 text-4xl mb-3">✓</div>
            <p className="text-gray-700">You're running the latest version!</p>
          </div>
        );
      case "error":
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <div className="text-red-500 text-4xl mb-3">✗</div>
            <p className="text-gray-700">Error checking for updates</p>
            <p className="text-red-500 mt-2">{errorMessage}</p>
          </div>
        );
      default:
        return (
          <div className="p-5 rounded-lg bg-gray-50">
            <p className="text-gray-600">Waiting to check for updates...</p>
          </div>
        );
    }
  };

  return (
    <div className="max-w-2xl mx-auto my-10 p-8 bg-white rounded-xl shadow-md">
      <button className="absolute top-2 right-2 text-gray-500 hover:text-gray-700" onClick={() => getCurrentWindow().close()}>
        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
      <h2 className="text-2xl font-bold text-gray-800 mb-6 text-center">Software Updates</h2>
      {renderContent()}
    </div>
  );
};

export default Updater;
