import { useState, useCallback, useEffect, useRef } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Toaster, toast } from "sonner";
import { Button } from "./components/ui/button";
import { X, MoreHorizontal, ChevronRight } from "lucide-react";

interface FilePreview {
  name: string;
  preview: string;
  type: 'file' | 'folder';
}

function App() {
  const [isDragging, setIsDragging] = useState(false);
  const [files, setFiles] = useState<FilePreview[]>([]);
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
            preview: '', // You might want to generate previews for these files
            type: 'file' as const
          }));
          handleNewFiles(newFiles);
        }
      });
    };

    setupFileListener();
  }, []);

  const handleNewFiles = (newFiles: FilePreview[]) => {
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
  };

  const handleDragEnter = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const droppedFiles = Array.from(e.dataTransfer.files);
    const newFiles: FilePreview[] = droppedFiles.map(file => ({
      name: file.name,
      preview: URL.createObjectURL(file),
      type: 'file'
    }));

    handleNewFiles(newFiles);
  }, []);

  const renderFilePreview = () => {
    if (files.length === 0) {
      return (
        <div className="flex items-center justify-center h-full">
          <p className="text-white text-lg font-medium">Drop files here</p>
        </div>
      );
    } else if (files.length === 1) {
      return (
        <div className="flex flex-col items-center justify-center h-full">
          <img src={files[0].preview || '/path/to/default/icon.png'} alt={files[0].name} className="w-32 h-32 object-cover rounded-lg" />
          <div className="mt-2 px-3 py-1 bg-gray-800 bg-opacity-50 rounded-full flex items-center">
            <span className="text-white text-sm truncate max-w-[150px]">{files[0].name}</span>
            <ChevronRight className="w-4 h-4 text-white ml-1" />
          </div>
        </div>
      );
    } else {
      return (
        <div className="flex flex-col items-center justify-center h-full">
          <div className="relative w-32 h-32">
            {files.slice(0, 3).map((file, index) => (
              <img
                key={file.name}
                src={file.preview || '/path/to/default/icon.png'}
                alt={file.name}
                className="absolute w-24 h-24 object-cover rounded-lg border-2 border-white shadow-md"
                style={{
                  top: `${index * 4}px`,
                  left: `${index * 4}px`,
                  zIndex: 3 - index
                }}
              />
            ))}
          </div>
          <div className="mt-2 px-3 py-1 bg-gray-800 bg-opacity-50 rounded-full flex items-center">
            <span className="text-white text-sm">{files.length} Files</span>
            <ChevronRight className="w-4 h-4 text-white ml-1" />
          </div>
        </div>
      );
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-900">
      <div
        className={`relative w-full h-screen max-w-7xl mx-auto p-4 rounded-3xl bg-gradient-to-b from-green-800 to-green-700 bg-opacity-80 backdrop-blur-sm shadow-lg ${
          isDragging ? 'ring-4 ring-white ring-opacity-60' : ''
        }`}
        onDragEnter={handleDragEnter}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <button className="absolute top-4 left-4 w-10 h-10 flex items-center justify-center rounded-full bg-gray-800 bg-opacity-50 text-white hover:bg-opacity-70 transition-colors">
          <X className="w-6 h-6" />
        </button>
        <button className="absolute top-4 right-4 w-10 h-10 flex items-center justify-center rounded-full bg-gray-800 bg-opacity-50 text-white hover:bg-opacity-70 transition-colors">
          <MoreHorizontal className="w-6 h-6" />
        </button>
        {renderFilePreview()}
      </div>
      <Toaster />
    </div>
  );
}

export default App;