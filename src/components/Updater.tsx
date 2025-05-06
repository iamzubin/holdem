import { check } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';

export function Updater() {
  const checkForUpdates = async () => {
    try {
      console.log('Checking for updates...');
      const update = await check();
      console.log('Raw update response:', update);
      
      if (update) {
        console.log('Update available:', update);
        const yes = await ask(
          `Update to ${update.version} is available!\n\nRelease notes: ${update.body}`,
          {
            title: 'Update Available',
            kind: 'info',
            okLabel: 'Update',
            cancelLabel: 'Cancel',
          }
        );
        if (yes) {
          await update.downloadAndInstall();
        }
      } else {
        console.log('No updates available');
        await message('You are on the latest version. Stay awesome!', {
          title: 'No Update Available',
          kind: 'info',
          okLabel: 'OK',
        });
      }
    } catch (error) {
      console.error('Update check failed:', error);
      console.error('Error details:', {
        name: error.name,
        message: error.message,
        stack: error.stack,
        cause: error.cause
      });
      await message('Update check failed', 'Failed to check for updates. Please try again later.');
    }
  };

  return (
    <button
      onClick={checkForUpdates}
      className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
    >
      Check for Updates
    </button>
  );
} 