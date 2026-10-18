import { useParams, Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, Download, Trash2, Heart, Folder } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { api } from "@/lib/api";
import { useAppStore } from "@/lib/store";
import { formatBytes, formatDate } from "@/lib/utils";

export function GameDetail() {
  const { id } = useParams<{ id: string }>();
  const activeDownloads = useAppStore((s) => s.activeDownloads);
  const setDownloadProgress = useAppStore((s) => s.setDownloadProgress);

  const { data: manifest } = useQuery({
    queryKey: ["game", id],
    queryFn: () => api.fetchGameManifest(id!),
    enabled: !!id,
  });

  const { data: local } = useQuery({
    queryKey: ["game-local", id],
    queryFn: () => api.getGameLocal(id!),
    enabled: !!id,
  });

  const downloadInfo = id ? activeDownloads[id] : undefined;

  if (!manifest) {
    return (
      <div className="container mx-auto p-6">
        <p className="text-muted-foreground">Đang tải...</p>
      </div>
    );
  }

  const handleInstall = async (version: string) => {
    if (!id) return;
    setDownloadProgress(id, { version, progress: 0, status: "starting" });
    try {
      await api.installGame(id, version);
    } catch (e) {
      console.error(e);
    }
  };

  const handleUninstall = async () => {
    if (!id) return;
    if (!confirm(`Gỡ bản việt hóa ${manifest.name}?`)) return;
    await api.uninstallGame(id);
  };

  const status = local?.status ?? "not_installed";

  return (
    <div className="container mx-auto p-6">
      <Link to="/library" className="mb-4 inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="h-4 w-4" />
        Quay lại thư viện
      </Link>

      <div className="grid gap-6 lg:grid-cols-[300px_1fr]">
        {/* Cover */}
        <div>
          <img
            src={manifest.coverImage}
            alt={manifest.name}
            className="w-full rounded-lg shadow-lg"
          />
          <Card className="mt-4 p-4">
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Nhà phát triển</span>
                <span>{manifest.developer}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Nhà phát hành</span>
                <span>{manifest.publisher}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Ngày phát hành</span>
                <span>{formatDate(manifest.releaseDate)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Trạng thái</span>
                <Badge variant={status === "installed" ? "success" : "secondary"}>
                  {status === "installed" ? "Đã cài" : status === "update_available" ? "Có cập nhật" : "Chưa cài"}
                </Badge>
              </div>
            </div>
          </Card>
        </div>

        {/* Main content */}
        <div className="space-y-6">
          <div>
            <div className="flex items-start justify-between gap-4">
              <div>
                <h1 className="text-3xl font-bold">{manifest.name}</h1>
                <p className="mt-2 text-sm text-muted-foreground">
                  {manifest.shortDescription}
                </p>
                <div className="mt-3 flex flex-wrap gap-1">
                  {manifest.genres.map((g) => (
                    <Badge key={g} variant="secondary">{g}</Badge>
                  ))}
                </div>
              </div>
              <div className="flex gap-2">
                {status !== "installed" ? (
                  <Button onClick={() => handleInstall(manifest.latestVersion)}>
                    <Download className="mr-2 h-4 w-4" />
                    {status === "update_available" ? "Cập nhật" : "Cài đặt"}
                  </Button>
                ) : (
                  <Button onClick={handleUninstall} variant="destructive">
                    <Trash2 className="mr-2 h-4 w-4" />
                    Gỡ cài đặt
                  </Button>
                )}
                <Button size="icon" variant="outline">
                  <Heart className="h-4 w-4" />
                </Button>
                <Button size="icon" variant="outline" onClick={() => api.openGameFolder(id!)}>
                  <Folder className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {/* Download progress */}
            {downloadInfo && (
              <Card className="mt-4 p-4">
                <div className="mb-2 flex items-center justify-between text-sm">
                  <span className="font-medium">Đang {downloadInfo.status}</span>
                  <span className="text-muted-foreground">{Math.round(downloadInfo.progress)}%</span>
                </div>
                <Progress value={downloadInfo.progress} />
              </Card>
            )}
          </div>

          {/* Description */}
          <Card className="p-6">
            <h2 className="mb-3 text-xl font-semibold">Mô tả</h2>
            <p className="whitespace-pre-line text-sm text-muted-foreground">
              {manifest.longDescription}
            </p>
          </Card>

          {/* Screenshots */}
          {manifest.screenshots.length > 0 && (
            <Card className="p-6">
              <h2 className="mb-3 text-xl font-semibold">Hình ảnh</h2>
              <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                {manifest.screenshots.map((url, i) => (
                  <img
                    key={i}
                    src={url}
                    alt={`Screenshot ${i + 1}`}
                    className="rounded-md"
                    loading="lazy"
                  />
                ))}
              </div>
            </Card>
          )}

          {/* Versions */}
          <Card className="p-6">
            <h2 className="mb-3 text-xl font-semibold">Phiên bản</h2>
            <div className="space-y-2">
              {manifest.versions.map((v) => (
                <div
                  key={v.version}
                  className="flex items-center justify-between rounded-md border p-3"
                >
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">v{v.version}</span>
                      {v.version === manifest.latestVersion && (
                        <Badge variant="success">Mới nhất</Badge>
                      )}
                      {v.isBeta && <Badge variant="outline">Beta</Badge>}
                      {v.isBroken && <Badge variant="destructive">Lỗi</Badge>}
                    </div>
                    <div className="mt-1 text-xs text-muted-foreground">
                      {formatDate(v.releasedAt)} · {formatBytes(v.sizeBytes)}
                    </div>
                    {v.changelog && (
                      <p className="mt-2 whitespace-pre-line text-sm text-muted-foreground">
                        {v.changelog}
                      </p>
                    )}
                  </div>
                  <Button
                    variant={v.version === manifest.latestVersion ? "default" : "outline"}
                    size="sm"
                    onClick={() => handleInstall(v.version)}
                    disabled={downloadInfo !== undefined}
                  >
                    <Download className="mr-1 h-3 w-3" />
                    Cài
                  </Button>
                </div>
              ))}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}