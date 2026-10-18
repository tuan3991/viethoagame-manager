import { useAppStore } from "@/lib/store";
import { Download, X } from "lucide-react";
import { Button } from "@/components/ui/button";

export function StatusBar() {
  const activeDownloads = useAppStore((s) => s.activeDownloads);
  const clearDownload = useAppStore((s) => s.clearDownload);
  const downloads = Object.entries(activeDownloads);

  if (downloads.length === 0) return null;

  return (
    <div className="border-t bg-card px-4 py-2">
      <div className="flex items-center gap-2 text-sm">
        <Download className="h-4 w-4 animate-pulse text-primary" />
        <span className="font-medium">{downloads.length} đang tải</span>
        <div className="ml-4 flex flex-1 gap-4 overflow-x-auto">
          {downloads.map(([gameId, info]) => (
            <div
              key={gameId}
              className="flex items-center gap-2 rounded border bg-background px-2 py-1"
            >
              <span className="font-mono text-xs">{gameId}</span>
              <span className="text-xs text-muted-foreground">{info.status}</span>
              <span className="text-xs font-medium">{Math.round(info.progress)}%</span>
              <Button
                size="sm"
                variant="ghost"
                className="h-5 w-5 p-0"
                onClick={() => clearDownload(gameId)}
              >
                <X className="h-3 w-3" />
              </Button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}