import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { GameStatus } from "@/types";

interface AppState {
  // Theme
  theme: "light" | "dark" | "system";
  setTheme: (theme: "light" | "dark" | "system") => void;

  // Library filter
  filter: {
    search: string;
    status: GameStatus | "all";
    launcher: "all" | "steam" | "epic" | "gog" | "manual";
    onlyFavorites: boolean;
  };
  setFilter: (filter: Partial<AppState["filter"]>) => void;

  // Active downloads
  activeDownloads: Record<string, { version: string; progress: number; status: string }>;
  setDownloadProgress: (gameId: string, info: { version: string; progress: number; status: string }) => void;
  clearDownload: (gameId: string) => void;

  // Settings cache
  installPath: string;
  setInstallPath: (path: string) => void;

  // Admin password (hashed, lưu local)
  adminUnlocked: boolean;
  setAdminUnlocked: (v: boolean) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      theme: "system",
      setTheme: (theme) => set({ theme }),

      filter: {
        search: "",
        status: "all",
        launcher: "all",
        onlyFavorites: false,
      },
      setFilter: (filter) =>
        set((state) => ({ filter: { ...state.filter, ...filter } })),

      activeDownloads: {},
      setDownloadProgress: (gameId, info) =>
        set((state) => ({
          activeDownloads: { ...state.activeDownloads, [gameId]: info },
        })),
      clearDownload: (gameId) =>
        set((state) => {
          const next = { ...state.activeDownloads };
          delete next[gameId];
          return { activeDownloads: next };
        }),

      installPath: "",
      setInstallPath: (path) => set({ installPath: path }),

      adminUnlocked: false,
      setAdminUnlocked: (v) => set({ adminUnlocked: v }),
    }),
    {
      name: "viethoagame-store",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        theme: state.theme,
        filter: state.filter,
        installPath: state.installPath,
      }),
    }
  )
);