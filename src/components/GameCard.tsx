import { Link } from "react-router-dom";
import { Download, CheckCircle2, AlertCircle, Heart } from "lucide-react";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { formatBytes } from "@/lib/utils";
import type { LocalGame, GameManifest } from "@/types";
import { cn } from "@/lib/utils";

interface GameCardProps {
  manifest: GameManifest;
  local?: LocalGame;
  favorite?: boolean;
  onFavorite?: () => void;
}

export function GameCard({ manifest, local, favorite, onFavorite }: GameCardProps) {
  const status = local?.status ?? "not_installed";
  const latestVersion = manifest.versions.find((v) => v.version === manifest.latestVersion);

  const statusConfig = {
    not_installed: { icon: Download, label: "Cài đặt", color: "bg-primary" },
    installed: { icon: CheckCircle2, label: "Đã cài", color: "bg-green-500" },
    update_available: { icon: AlertCircle, label: "Có bản mới", color: "bg-orange-500" },
    broken: { icon: AlertCircle, label: "Lỗi", color: "bg-destructive" },
  };

  const StatusIcon = statusConfig[status].icon;

  return (
    <Card className="group overflow-hidden transition-all hover:shadow-lg">
      <Link to={`/game/${manifest.id}`} className="block">
        <div className="relative aspect-[3/2] bg-muted">
          {manifest.coverImage ? (
            <img
              src={manifest.coverImage}
              alt={manifest.name}
              className="h-full w-full object-cover"
              loading="lazy"
            />
          ) : (
            <div className="flex h-full w-full items-center justify-center">
              <span className="text-4xl font-bold text-muted-foreground">
                {manifest.name.charAt(0)}
              </span>
            </div>
          )}
          {status === "update_available" && (
            <Badge variant="destructive" className="absolute right-2 top-2">
              NEW
            </Badge>
          )}
          {favorite && (
            <Heart className="absolute left-2 top-2 h-5 w-5 fill-red-500 text-red-500" />
          )}
        </div>
      </Link>
      <div className="p-3">
        <Link to={`/game/${manifest.id}`}>
          <h3 className="line-clamp-1 font-semibold transition-colors group-hover:text-primary">
            {manifest.name}
          </h3>
        </Link>
        <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
          <span>v{manifest.latestVersion}</span>
          {latestVersion && (
            <>
              <span>·</span>
              <span>{formatBytes(latestVersion.sizeBytes)}</span>
            </>
          )}
        </div>
        <div className="mt-3 flex items-center justify-between gap-2">
          <Badge
            variant={
              status === "installed" ? "success" :
              status === "update_available" ? "destructive" :
              status === "broken" ? "destructive" : "secondary"
            }
            className="gap-1"
          >
            <StatusIcon className="h-3 w-3" />
            {statusConfig[status].label}
          </Badge>
          {onFavorite && (
            <Button
              size="sm"
              variant="ghost"
              className={cn("h-7 w-7 p-0", favorite && "text-red-500")}
              onClick={(e) => {
                e.preventDefault();
                onFavorite();
              }}
            >
              <Heart className={cn("h-4 w-4", favorite && "fill-current")} />
            </Button>
          )}
        </div>
      </div>
    </Card>
  );
}