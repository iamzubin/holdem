import { Copy, X } from "lucide-react";
import { Button } from "./ui/button";
import { FilePreview } from "@/types";

interface ShelfItemProps {
  file: FilePreview;
  onClear: (id: number) => void;
  onCopyLink: (file: FilePreview) => void;
}

export function ShelfItem({ file, onClear, onCopyLink }: ShelfItemProps) {
  return (
    <div className="group relative flex items-center justify-between p-2 rounded-md hover:bg-secondary/50 transition-colors w-full border border-transparent hover:border-border">
      <div className="flex items-center space-x-3 overflow-hidden">
        {file.preview ? (
          <img src={file.preview} alt={file.name} className="w-8 h-8 rounded object-cover" />
        ) : (
          <div className="w-8 h-8 rounded bg-muted flex items-center justify-center text-xs">
            {file.icon}
          </div>
        )}
        <span className="text-sm truncate max-w-[150px] font-medium">{file.name}</span>
      </div>
      
      {/* Actions (visible on hover) */}
      <div className="opacity-0 group-hover:opacity-100 flex items-center space-x-1 transition-opacity">
        <Button 
          variant="ghost" 
          size="icon" 
          className="h-7 w-7 rounded-sm" 
          onClick={(e) => { e.stopPropagation(); onCopyLink(file); }} 
          title="Copy Link"
        >
          <Copy className="h-3.5 w-3.5" />
        </Button>
        <Button 
          variant="ghost" 
          size="icon" 
          className="h-7 w-7 rounded-sm text-destructive hover:bg-destructive/10" 
          onClick={(e) => { e.stopPropagation(); onClear(file.id); }} 
          title="Clear"
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
