import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Search, Filter } from "lucide-react";
import { Input } from "@/components/ui/input";
import { GameCard } from "@/components/GameCard";
import { api } from "@/lib/api";
import { useAppStore } from "@/lib/store";
import type { GameManifest, LocalGame } from "@/types";

export function Library() {
  const filter = useAppStore((s) => s.filter);
  const setFilter = useAppStore((s) => s.setFilter);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());

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
      </div>

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
            />
          ))}
        </div>
      )}
    </div>
  );
}