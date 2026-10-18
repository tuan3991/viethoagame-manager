/**
 * Types cho dữ liệu từ Google Drive
 */

export interface Manifest {
  version: number;
  lastUpdated: string;
  games: ManifestEntry[];
}

export interface ManifestEntry {
  id: string;
  name: string;
  manifestUrl: string;
}

export interface GameManifest {
  id: string;
  name: string;
  shortDescription: string;
  longDescription: string;
  developer: string;
  publisher: string;
  releaseDate: string;
  genres: string[];
  tags: string[];
  coverImage: string;
  screenshots: string[];
  videoUrl?: string;
  systemRequirements: {
    min: string;
    rec: string;
  };
  supportedEngines: string[];
  latestVersion: string;
  versions: GameVersion[];
  detect: GameDetection;
  install: InstallConfig;
  uninstall?: UninstallConfig;
  stats?: GameStats;
}

export interface GameVersion {
  version: string;
  releasedAt: string;
  sizeBytes: number;
  downloadUrl: string;
  checksum: string;
  changelog?: string;
  isBeta?: boolean;
  isBroken?: boolean;
}

export interface GameDetection {
  launchers: Array<"steam" | "epic" | "gog" | "manual">;
  steamAppId?: string;
  epicAppName?: string;
  gogAppId?: string;
  executableRelative?: string;
  customFolders?: string[];
}

export interface InstallConfig {
  files: InstallRule[];
  preInstall?: InstallHook[];
  postInstall?: InstallHook[];
  notes?: string;
}

export interface InstallRule {
  source: string;
  target: string;
  backupOriginal: boolean;
  overwrite?: boolean;
}

export type InstallHook =
  | { type: "kill_process"; processNames: string[] }
  | { type: "verify"; checkFiles: string[] }
  | { type: "open_url"; url: string };

export interface UninstallConfig {
  restoreBackup: boolean;
  removeAddedFiles?: string[];
}

export interface GameStats {
  downloads: number;
  activeInstalls: number;
  averageRating: number;
  ratingCount: number;
}

/**
 * Types cho local DB
 */
export type GameStatus =
  | "not_installed"
  | "installed"
  | "update_available"
  | "broken";

export interface LocalGame {
  id: string;
  name: string;
  version: string;
  installed_version: string | null;
  install_path: string | null;
  installed_at: number | null;
  last_checked: number;
  status: GameStatus;
}

export interface InstallHistoryEntry {
  id: number;
  game_id: string;
  version: string;
  action: "install" | "uninstall" | "update";
  success: boolean;
  error_message: string | null;
  created_at: number;
}

export interface BackupEntry {
  id: number;
  game_id: string;
  version: string;
  file_path: string;
  original_path: string;
  file_hash: string;
  created_at: number;
}

/**
 * Download progress event từ backend
 */
export interface DownloadProgress {
  gameId: string;
  version: string;
  downloadedBytes: number;
  totalBytes: number;
  speedBytesPerSec: number;
  status: "downloading" | "verifying" | "extracting" | "installing" | "done" | "failed";
  message?: string;
}

/**
 * Admin types
 */
export interface AdminAnalytics {
  totalDownloads: number;
  activeInstalls: number;
  uniqueUsers: number;
  averageRating: number;
  downloadsThisWeek: number;
  downloadsTrend: string;
  topGames: Array<{ id: string; name: string; downloads: number }>;
  downloadsByDay: Array<{ date: string; downloads: number }>;
}

export interface BugReport {
  id: string;
  gameId: string;
  version: string;
  userId: string;
  title: string;
  description: string;
  status: "open" | "investigating" | "resolved" | "closed";
  createdAt: number;
}

export interface Comment {
  id: string;
  gameId: string;
  userId: string;
  rating: number;
  content: string;
  status: "pending" | "approved" | "rejected";
  createdAt: number;
}

export interface UserRequest {
  id: string;
  gameName: string;
  votes: number;
  requestedBy: string;
  createdAt: number;
}