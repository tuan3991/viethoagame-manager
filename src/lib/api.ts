import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type {
  Manifest,
  GameManifest,
  GameVersion,
  LocalGame,
  DownloadProgress,
  GameStatus,
  AdminAnalytics,
  BugReport,
  Comment,
  UserRequest,
} from "@/types";

/**
 * Wrapper cho Tauri commands
 */

export const api = {
  // === Manifest / Library ===
  fetchManifest: () => invoke<Manifest>("fetch_manifest"),
  fetchGameManifest: (gameId: string) =>
    invoke<GameManifest>("fetch_game_manifest", { gameId }),

  // === Scanner ===
  scanInstalledGames: () => invoke<LocalGame[]>("scan_installed_games"),
  detectGamePath: (gameId: string) =>
    invoke<string | null>("detect_game_path", { gameId }),
  setCustomGamePath: (gameId: string, path: string) =>
    invoke<void>("set_custom_game_path", { gameId, path }),

  // === Library local ===
  getLibrary: () => invoke<LocalGame[]>("get_library"),
  getGameLocal: (gameId: string) =>
    invoke<LocalGame | null>("get_game_local", { gameId }),
  setGameStatus: (gameId: string, status: GameStatus) =>
    invoke<void>("set_game_status", { gameId, status }),

  // === Install ===
  installGame: (gameId: string, version: string) =>
    invoke<void>("install_game", { gameId, version }),
  uninstallGame: (gameId: string) =>
    invoke<void>("uninstall_game", { gameId }),
  cancelInstall: (gameId: string) =>
    invoke<void>("cancel_install", { gameId }),

  // === Favorites ===
  toggleFavorite: (gameId: string) =>
    invoke<boolean>("toggle_favorite", { gameId }),
  getFavorites: () => invoke<string[]>("get_favorites"),

  // === Settings ===
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) =>
    invoke<void>("set_setting", { key, value }),

  // === Admin ===
  adminUpsertGame: (gameId: string, name: string, manifestUrl: string) =>
    invoke<void>("admin_upsert_game", { gameId, name, manifestUrl }),
  adminDeleteGame: (gameId: string) =>
    invoke<void>("admin_delete_game", { gameId }),
  adminAddVersion: (gameId: string, version: GameVersion) =>
    invoke<void>("admin_add_version", { gameId, version }),
  adminGetAnalytics: () => invoke<AdminAnalytics>("admin_get_analytics"),
  adminGetReports: () => invoke<BugReport[]>("admin_get_reports"),
  adminGetComments: () => invoke<Comment[]>("admin_get_comments"),
  adminGetRequests: () => invoke<UserRequest[]>("admin_get_requests"),

  // === App ===
  getAppVersion: () => invoke<string>("get_app_version"),
  openGameFolder: (gameId: string) =>
    invoke<void>("open_game_folder", { gameId }),
  launchGame: (gameId: string) =>
    invoke<void>("launch_game", { gameId }),
};

/**
 * Subscribe download progress event
 */
export function onDownloadProgress(
  callback: (progress: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) =>
    callback(event.payload)
  );
}