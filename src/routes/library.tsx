import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Search, Filter, Sparkles, CheckCircle2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { GameCard } from "@/components/GameCard";
import { api } from "@/lib/api";
import { useAppStore } from "@/lib/store";
import type { GameManifest, LocalGame, AutoSetupResult } from "@/types";

export function Library() {
  const filter = useAppStore((s) => s.filter);
  const setFilter = useAppStore((s) => s.setFilter);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());
  const queryClient = useQueryClient();
  const [autoSetupDone, setAutoSetupDone] = useState(false);
  const [autoSetupResult, setAutoSetupResult] = useState<AutoSetupResult | null>(null);
  const [autoSetupRunning, setAutoSetupRunning] = useState(false);

  const { data: manifest } = useQuery({
    queryKey: ["manifest"],
    queryFn: api.fetchManifest,
  });

  const { data: localGames } = useQuery({
    queryKey: ["library"],
    queryFn: api.getLibrary,
  });

  // Load favorites
  useEffect(() => {
    api.getFavorites().then((f) => setFavorites(new Set(f)));
  }, []);

  // Auto-detect on first mount: nếu localGames trống → gọi auto_setup_games
  useEffect(() => {
    if (autoSetupDone || autoSetupRunning) return;
    if (localGames === undefined) return; // chưa load xong

    // Nếu chưa có game nào detect được → chạy auto-setup
    const hasAnyPath = localGames.some((g) => g.install_path);
    if (hasAnyPath) {
      setAutoSetupDone(true);
      return;
    }

    setAutoSetupRunning(true);
    api.autoSetupGames()
      .then((res) => {
        setAutoSetupResult(res);
        setAutoSetupDone(true);
        queryClient.invalidateQueries({ queryKey: ["library"] });
      })
      .catch((err) => {
        console.error("Auto-setup failed:", err);
        setAutoSetupDone(true);
      })
      .finally(() => setAutoSetupRunning(false));
  }, [localGames, autoSetupDone, autoSetupRunning, queryClient]);

  // Cho phép user re-run auto-setup manually
  const runAutoSetup = async () => {
    setAutoSetupRunning(true);
    try {
      const res = await api.autoSetupGames();
      setAutoSetupResult(res);
      queryClient.invalidateQueries({ queryKey: ["library"] });
    } catch (err) {
      console.error(err);
    } finally {
      setAutoSetupRunning(false);
    }
  };

  // Fetch full game manifests in parallel
  const { data: gameManifests } = useQuery({
    queryKey: ["game-manifests", manifest?.games.map((g) => g.id)],
    queryFn: async () => {
      if (!manifest) return [] as GameManifest[];
      return Promise.all(
        manifest.games.map((g) => api.fetchGameManifest(g.id).catch(() => null))
      ).then((results) => results.filter((r): r is GameManifest => r !== null));
    },
    enabled: !!manifest,
  });

  const localById = new Map<string, LocalGame>(
    (localGames ?? []).map((g) => [g.id, g])
  );

  // Lookup entry cover/avatar by id
  const entryById = new Map<string, { coverImage?: string; avatarImage?: string }>(
    (manifest?.games ?? []).map((g) => [g.id, { coverImage: g.coverImage, avatarImage: g.avatarImage }])
  );

  const filtered = (gameManifests ?? []).filter((game) => {
    if (filter.search && !game.name.toLowerCase().includes(filter.search.toLowerCase())) {
      return false;
    }
    if (filter.status !== "all") {
      const local = localById.get(game.id);
      if ((local?.status ?? "not_installed") !== filter.status) return false;
    }
    if (filter.onlyFavorites && !favorites.has(game.id)) return false;
    return true;
  });

  const handleFavorite = async (gameId: string) => {
    await api.toggleFavorite(gameId);
    const next = await api.getFavorites();
    setFavorites(new Set(next));
  };

  return (
    <div className="container mx-auto p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold">Thư viện</h2>
          <p className="text-sm text-muted-foreground">
            {filtered.length} / {gameManifests?.length ?? 0} game
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={runAutoSetup}
          disabled={autoSetupRunning}
          className="gap-2"
        >
          <Sparkles className={`h-4 w-4 ${autoSetupRunning ? "animate-pulse" : ""}`} />
          {autoSetupRunning ? "Đang quét..." : "Quét tự động"}
        </Button>
      </div>

      {/* Auto-setup result banner */}
      {autoSetupResult && (autoSetupResult.matched.length > 0 || autoSetupResult.skipped.length > 0) && (
        <div className="mb-6 rounded-lg border bg-card p-4">
          <div className="flex items-start gap-3">
            <Sparkles className="h-5 w-5 text-primary mt-0.5" />
            <div className="flex-1 space-y-2">
              <div className="font-semibold">Kết quả quét tự động</div>
              <div className="text-sm text-muted-foreground">
                Quét {autoSetupResult.totalScanned} game trên máy (Steam/Epic/GOG).
                Tự động nhận diện <span className="font-semibold text-foreground">{autoSetupResult.matched.length}</span> game.
              </div>
              {autoSetupResult.matched.length > 0 && (
                <div className="flex flex-wrap gap-1.5">
                  {autoSetupResult.matched.map((m) => (
                    <span
                      key={m.gameId}
                      className="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2.5 py-0.5 text-xs text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                    >
                      <CheckCircle2 className="h-3 w-3" />
                      {m.name}
                    </span>
                  ))}
                </div>
              )}
              {autoSetupResult.skipped.length > 0 && (
                <div className="text-xs text-muted-foreground">
                  Không tìm thấy: {autoSetupResult.skipped.map((s) => s.gameId).join(", ")}
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Filter bar */}
      <div className="mb-6 flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Tìm kiếm game..."
            value={filter.search}
            onChange={(e) => setFilter({ search: e.target.value })}
            className="pl-9"
          />
        </div>
        <select
          value={filter.status}
          onChange={(e) => setFilter({ status: e.target.value as any })}
          className="h-10 rounded-md border bg-background px-3 text-sm"
        >
          <option value="all">Tất cả trạng thái</option>
          <option value="not_installed">Chưa cài</option>
          <option value="installed">Đã cài</option>
          <option value="update_available">Có cập nhật</option>
          <option value="broken">Lỗi</option>
        </select>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={filter.onlyFavorites}
            onChange={(e) => setFilter({ onlyFavorites: e.target.checked })}
          />
          Yêu thích
        </label>
      </div>

      {/* Grid */}
      {filtered.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <Filter className="mb-4 h-12 w-12 text-muted-foreground" />
          <p className="text-lg text-muted-foreground">Không tìm thấy game nào</p>
        </div>
      ) : (
        <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
          {filtered.map((game) => (
            <GameCard
              key={game.id}
              manifest={game}
              local={localById.get(game.id)}
              favorite={favorites.has(game.id)}
              onFavorite={() => handleFavorite(game.id)}
              entryCover={entryById.get(game.id)?.coverImage}
              entryAvatar={entryById.get(game.id)?.avatarImage}
            />
          ))}
        </div>
      )}
    </div>
  );
}