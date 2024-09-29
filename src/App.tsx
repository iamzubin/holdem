import { useState, useEffect, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Toaster, toast } from "sonner";
import { Card, CardContent } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { FileIcon, FolderIcon } from "lucide-react";

import "./App.css";

function App() {
  const [files, setFiles] = useState<{ name: string; type: 'file' | 'folder' }[]>([]);
  const [draggedItem, setDraggedItem] = useState<{ name: string; type: 'file' | 'folder' } | null>(null);
  const listenerSetup = useRef(false);

  useEffect(() => {
    if (listenerSetup.current) return;
    listenerSetup.current = true;

    const setupFileListener = async () => {
      const webview = await getCurrentWebview();
      await webview.onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          const newFiles = event.payload.paths.map(path => ({
            name: path.split('/').pop() || '',
            type: 'file' as const
          }));
          setFiles(prevFiles => {
            const uniqueNewFiles = newFiles.filter(newFile => 
              !prevFiles.some(existingFile => existingFile.name === newFile.name)
            );
            if (uniqueNewFiles.length > 0) {
              toast.success(`Added ${uniqueNewFiles.length} new file(s)`);
            } else {
              toast.info("No new files added");
            }
            return [...prevFiles, ...uniqueNewFiles];
          });
        }
      });
    };

    setupFileListener();
  }, []);

  return (
    <div className="container mx-auto p-4">
      <Card className="w-full h-[calc(100vh-2rem)]">
        <CardContent className="p-6">
          <ScrollArea className="h-[calc(100vh-10rem)]">
            <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {files.map((item, index) => (
                <Button
                  key={index}
                  variant="outline"
                  className="h-24 flex flex-col items-center justify-center"
                  draggable
                  onDragStart={() => setDraggedItem(item)}
                  onDragEnd={() => setDraggedItem(null)}
                >
                  {item.type === 'folder' ? <FolderIcon className="h-8 w-8 mb-2" /> : <FileIcon className="h-8 w-8 mb-2" />}
                  <span className="text-sm truncate w-full text-center">{item.name}</span>
                </Button>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
      <div className="mt-4 text-sm text-gray-500">
        {files.length} item(s)
      </div>
      <Toaster />
    </div>
  );
}

export default App;
