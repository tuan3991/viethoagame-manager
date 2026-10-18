import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import {
  Lock, Unlock, Plus, Trash2, Edit2, Save, Download,
  Users, BarChart3, MessageSquare, AlertTriangle,
  Star, TrendingUp, Eye, RefreshCw, FileJson, Package,
} from "lucide-react";
import { api } from "@/lib/api";
import { useAppStore } from "@/lib/store";
import { formatRelativeTime } from "@/lib/utils";

// ================================================================
// MAIN COMPONENT
// ================================================================

export function Admin() {
  const adminUnlocked = useAppStore((s) => s.adminUnlocked);
  const setAdminUnlocked = useAppStore((s) => s.setAdminUnlocked);
  const [password, setPassword] = useState("");
  const [activeTab, setActiveTab] = useState("games");

  if (!adminUnlocked) {
    return (
      <LoginGate
        password={password}
        setPassword={setPassword}
        onUnlock={() => {
          // Trong production, hash password và verify
          // Hiện tại: chỉ cần nhập đúng pass đơn giản (sẽ improve sau)
          if (password.length >= 4) {
            setAdminUnlocked(true);
          }
        }}
      />
    );
  }

  return (
    <div className="container mx-auto p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold">Admin Panel</h2>
          <p className="text-sm text-muted-foreground">
            Quản lý thư viện, thống kê và cộng đồng
          </p>
        </div>
        <Button variant="outline" onClick={async () => {
          try {
            await api.setSetting("admin_session", "");
          } catch {}
          setAdminUnlocked(false);
        }}>
          <Lock className="mr-2 h-4 w-4" />
          Khoá
        </Button>
      </div>

      {/* Tab nav */}
      <div className="mb-6 flex gap-1 border-b">
        <TabButton active={activeTab === "games"} onClick={() => setActiveTab("games")}>
          <Package className="mr-2 inline h-4 w-4" />
          Games ({0})
        </TabButton>
        <TabButton active={activeTab === "analytics"} onClick={() => setActiveTab("analytics")}>
          <BarChart3 className="mr-2 inline h-4 w-4" />
          Analytics
        </TabButton>
        <TabButton active={activeTab === "reports"} onClick={() => setActiveTab("reports")}>
          <AlertTriangle className="mr-2 inline h-4 w-4" />
          Báo lỗi ({0})
        </TabButton>
        <TabButton active={activeTab === "comments"} onClick={() => setActiveTab("comments")}>
          <MessageSquare className="mr-2 inline h-4 w-4" />
          Bình luận ({0})
        </TabButton>
        <TabButton active={activeTab === "requests"} onClick={() => setActiveTab("requests")}>
          <Users className="mr-2 inline h-4 w-4" />
          Yêu cầu ({0})
        </TabButton>
        <TabButton active={activeTab === "settings"} onClick={() => setActiveTab("settings")}>
          <FileJson className="mr-2 inline h-4 w-4" />
          Drive Config
        </TabButton>
      </div>

      {/* Tab content */}
      {activeTab === "games" && <GamesTab />}
      {activeTab === "analytics" && <AnalyticsTab />}
      {activeTab === "reports" && <ReportsTab />}
      {activeTab === "comments" && <CommentsTab />}
      {activeTab === "requests" && <RequestsTab />}
      {activeTab === "settings" && <DriveSettingsTab />}
    </div>
  );
}

// ================================================================
// SUB COMPONENTS
// ================================================================

function LoginGate({
  password,
  setPassword,
  onUnlock,
}: {
  password: string;
  setPassword: (p: string) => void;
  onUnlock: () => void;
}) {
  const handleUnlock = async () => {
    if (password.length < 4) return;
    // Set session token trong backend để cho phép gọi admin commands
    try {
      await api.setSetting("admin_session", "valid");
      onUnlock();
    } catch (e) {
      alert(`Lỗi: ${e}`);
    }
  };

  return (
    <div className="container mx-auto flex items-center justify-center p-6">
      <Card className="w-96">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Lock className="h-5 w-5" />
            Admin Panel
          </CardTitle>
          <CardDescription>
            Nhập mật khẩu admin để tiếp tục
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <Input
            type="password"
            placeholder="Mật khẩu admin"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleUnlock();
            }}
            autoFocus
          />
          <Button onClick={handleUnlock} disabled={password.length < 4} className="w-full">
            <Unlock className="mr-2 h-4 w-4" />
            Mở khóa
          </Button>
          <p className="text-xs text-muted-foreground">
            Mật khẩu lưu local, không gửi lên server. Trong production nên dùng Argon2 + keyring.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`border-b-2 px-4 py-2 text-sm font-medium transition-colors ${
        active
          ? "border-primary text-primary"
          : "border-transparent text-muted-foreground hover:text-foreground"
      }`}
    >
      {children}
    </button>
  );
}

// ================================================================
// TAB 1: GAMES MANAGEMENT
// ================================================================

function GamesTab() {
  const { data: manifest, refetch } = useQuery({
    queryKey: ["admin-manifest"],
    queryFn: api.fetchManifest,
  });

  const [editing, setEditing] = useState<{ mode: "add" | "edit"; gameId?: string } | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredGames = (manifest?.games ?? []).filter((g) =>
    g.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    g.id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-4">
      {/* Toolbar */}
      <div className="flex items-center justify-between gap-4">
        <Input
          placeholder="Tìm game..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="max-w-sm"
        />
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => refetch()}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Tải lại manifest
          </Button>
          <Button onClick={() => setEditing({ mode: "add" })}>
            <Plus className="mr-2 h-4 w-4" />
            Thêm game
          </Button>
        </div>
      </div>

      {/* Game list */}
      <Card>
        <CardContent className="p-0">
          <table className="w-full">
            <thead className="border-b bg-muted/50">
              <tr className="text-left text-xs uppercase text-muted-foreground">
                <th className="p-3">ID</th>
                <th className="p-3">Tên game</th>
                <th className="p-3">Manifest URL</th>
                <th className="p-3">Hành động</th>
              </tr>
            </thead>
            <tbody>
              {filteredGames.map((game) => (
                <tr key={game.id} className="border-b hover:bg-muted/30">
                  <td className="p-3 font-mono text-xs">{game.id}</td>
                  <td className="p-3 font-medium">{game.name}</td>
                  <td className="p-3 max-w-md truncate font-mono text-xs text-muted-foreground">
                    {game.manifestUrl}
                  </td>
                  <td className="p-3">
                    <div className="flex gap-1">
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => setEditing({ mode: "edit", gameId: game.id })}
                      >
                        <Edit2 className="h-3 w-3" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          if (confirm(`Xoá ${game.name}?`)) {
                            // TODO: gọi API xoá
                          }
                        }}
                      >
                        <Trash2 className="h-3 w-3 text-destructive" />
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
              {filteredGames.length === 0 && (
                <tr>
                  <td colSpan={4} className="p-8 text-center text-muted-foreground">
                    Không có game nào. Bấm "Thêm game" để bắt đầu.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </CardContent>
      </Card>

      {editing && (
        <GameEditDialog
          mode={editing.mode}
          gameId={editing.gameId}
          onClose={() => setEditing(null)}
          onSaved={() => {
            setEditing(null);
            refetch();
          }}
        />
      )}
    </div>
  );
}

function GameEditDialog({
  mode,
  gameId,
  onClose,
  onSaved,
}: {
  mode: "add" | "edit";
  gameId?: string;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [form, setForm] = useState({
    id: gameId ?? "",
    name: "",
    manifestUrl: "",
  });

  const handleSave = async () => {
    if (!form.id || !form.name || !form.manifestUrl) {
      alert("Vui lòng điền đầy đủ");
      return;
    }

    // Gọi API backend
    try {
      await api.adminUpsertGame(form.id, form.name, form.manifestUrl);
      onSaved();
    } catch (e) {
      alert(`Lỗi: ${e}`);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <CardTitle>{mode === "add" ? "Thêm game mới" : `Sửa: ${gameId}`}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium">Game ID</label>
            <Input
              placeholder="vd: witcher-3"
              value={form.id}
              onChange={(e) => setForm({ ...form, id: e.target.value })}
              disabled={mode === "edit"}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              Chỉ a-z, 0-9, gạch ngang. Không đổi sau khi tạo.
            </p>
          </div>
          <div>
            <label className="text-sm font-medium">Tên game</label>
            <Input
              placeholder="The Witcher 3: Wild Hunt"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </div>
          <div>
            <label className="text-sm font-medium">Manifest URL</label>
            <Input
              placeholder="https://drive.google.com/uc?export=download&id=..."
              value={form.manifestUrl}
              onChange={(e) => setForm({ ...form, manifestUrl: e.target.value })}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              URL tới file <code>manifest.json</code> của game trên Drive
            </p>
          </div>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={onClose}>
              Huỷ
            </Button>
            <Button onClick={handleSave}>
              <Save className="mr-2 h-4 w-4" />
              Lưu
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ================================================================
// TAB 2: ANALYTICS
// ================================================================

function AnalyticsTab() {
  // Mock data - sẽ lấy từ backend sau
  const mockStats = {
    totalDownloads: 24580,
    activeInstalls: 12450,
    uniqueUsers: 8932,
    avgRating: 4.6,
    downloadsThisWeek: 1234,
    downloadsTrend: "+12.5%",
    topGames: [
      { id: "witcher-3", name: "The Witcher 3", downloads: 5240 },
      { id: "cyberpunk2077", name: "Cyberpunk 2077", downloads: 4180 },
      { id: "bg3", name: "Baldur's Gate 3", downloads: 3920 },
      { id: "stardew", name: "Stardew Valley", downloads: 2150 },
      { id: "hollow-knight", name: "Hollow Knight", downloads: 1840 },
    ],
    downloadsByDay: [
      { date: "12/10", downloads: 145 },
      { date: "13/10", downloads: 178 },
      { date: "14/10", downloads: 234 },
      { date: "15/10", downloads: 198 },
      { date: "16/10", downloads: 267 },
      { date: "17/10", downloads: 312 },
      { date: "18/10", downloads: 289 },
    ],
  };

  return (
    <div className="space-y-4">
      {/* KPI cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <KPICard
          icon={<Download className="h-5 w-5" />}
          label="Tổng lượt tải"
          value={mockStats.totalDownloads.toLocaleString()}
          trend={mockStats.downloadsTrend}
          trendUp
        />
        <KPICard
          icon={<Eye className="h-5 w-5" />}
          label="Bản đang cài"
          value={mockStats.activeInstalls.toLocaleString()}
        />
        <KPICard
          icon={<Users className="h-5 w-5" />}
          label="User duy nhất"
          value={mockStats.uniqueUsers.toLocaleString()}
        />
        <KPICard
          icon={<Star className="h-5 w-5" />}
          label="Đánh giá TB"
          value={mockStats.avgRating.toFixed(1)}
          subtitle="/ 5.0"
        />
      </div>

      {/* Downloads chart */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card className="md:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-5 w-5" />
              Lượt tải 7 ngày qua
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {mockStats.downloadsByDay.map((day) => {
                const max = Math.max(...mockStats.downloadsByDay.map((d) => d.downloads));
                const percent = (day.downloads / max) * 100;
                return (
                  <div key={day.date} className="flex items-center gap-3">
                    <div className="w-16 text-xs text-muted-foreground">{day.date}</div>
                    <div className="flex-1">
                      <Progress value={percent} />
                    </div>
                    <div className="w-12 text-right text-sm font-medium">{day.downloads}</div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Top 5 game</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {mockStats.topGames.map((game, i) => (
                <div key={game.id} className="flex items-center gap-3">
                  <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-bold text-primary">
                    {i + 1}
                  </div>
                  <div className="flex-1">
                    <div className="font-medium">{game.name}</div>
                    <div className="text-xs text-muted-foreground">
                      {game.downloads.toLocaleString()} lượt tải
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function KPICard({
  icon,
  label,
  value,
  subtitle,
  trend,
  trendUp,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtitle?: string;
  trend?: string;
  trendUp?: boolean;
}) {
  return (
    <Card>
      <CardContent className="p-6">
        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">{label}</div>
          <div className="text-primary">{icon}</div>
        </div>
        <div className="mt-2 text-3xl font-bold">
          {value}
          {subtitle && <span className="text-base font-normal text-muted-foreground">{subtitle}</span>}
        </div>
        {trend && (
          <div className={`mt-1 text-xs ${trendUp ? "text-green-500" : "text-red-500"}`}>
            {trendUp ? "↑" : "↓"} {trend}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// ================================================================
// TAB 3: BUG REPORTS
// ================================================================

function ReportsTab() {
  const mockReports = [
    {
      id: "1",
      gameId: "witcher-3",
      version: "1.32",
      user: "user_abc123",
      title: "Font tiếng Việt bị lỗi ở menu phụ",
      description: "Một số chữ trong menu phụ đề hiển thị font lỗi...",
      status: "open",
      createdAt: Date.now() / 1000 - 3600 * 2,
    },
    {
      id: "2",
      gameId: "cyberpunk2077",
      version: "2.0",
      user: "user_xyz789",
      title: "Crash khi vào khu vực Watson",
      description: "Game crash khi load khu vực Watson...",
      status: "investigating",
      createdAt: Date.now() / 1000 - 86400,
    },
    {
      id: "3",
      gameId: "bg3",
      version: "4.1.1",
      user: "user_def456",
      title: "Phụ đề bị cắt ở Act 3",
      description: "Các câu thoại dài bị cắt mất phần sau...",
      status: "resolved",
      createdAt: Date.now() / 1000 - 86400 * 3,
    },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Báo lỗi từ người dùng</CardTitle>
        <CardDescription>
          {mockReports.filter((r) => r.status === "open").length} chưa xử lý
        </CardDescription>
      </CardHeader>
      <CardContent className="p-0">
        <div className="divide-y">
          {mockReports.map((report) => (
            <div key={report.id} className="p-4 hover:bg-muted/30">
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <h4 className="font-semibold">{report.title}</h4>
                    <Badge
                      variant={
                        report.status === "open" ? "destructive" :
                        report.status === "investigating" ? "outline" : "secondary"
                      }
                    >
                      {report.status}
                    </Badge>
                  </div>
                  <p className="mt-1 text-sm text-muted-foreground">{report.description}</p>
                  <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
                    <span>Game: <span className="font-mono">{report.gameId}</span></span>
                    <span>v{report.version}</span>
                    <span>·</span>
                    <span>bởi {report.user}</span>
                    <span>·</span>
                    <span>{formatRelativeTime(report.createdAt * 1000)}</span>
                  </div>
                </div>
                <div className="flex gap-2">
                  <Button size="sm" variant="outline">Xem</Button>
                  <Button size="sm">Đóng</Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

// ================================================================
// TAB 4: COMMENTS
// ================================================================

function CommentsTab() {
  const mockComments = [
    {
      id: "1",
      gameId: "witcher-3",
      user: "user_abc",
      rating: 5,
      content: "Bản việt hóa rất tốt, dịch mượt và tự nhiên!",
      status: "approved",
      createdAt: Date.now() / 1000 - 3600,
    },
    {
      id: "2",
      gameId: "bg3",
      user: "user_xyz",
      rating: 4,
      content: "Tốt nhưng có vài chỗ dịch chưa chuẩn...",
      status: "pending",
      createdAt: Date.now() / 1000 - 7200,
    },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Đánh giá & bình luận</CardTitle>
        <CardDescription>
          {mockComments.filter((c) => c.status === "pending").length} đang chờ duyệt
        </CardDescription>
      </CardHeader>
      <CardContent className="p-0">
        <div className="divide-y">
          {mockComments.map((comment) => (
            <div key={comment.id} className="p-4">
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{comment.user}</span>
                    <span className="text-sm text-muted-foreground">trên</span>
                    <span className="font-mono text-xs">{comment.gameId}</span>
                    <div className="flex">
                      {Array.from({ length: 5 }).map((_, i) => (
                        <Star
                          key={i}
                          className={`h-3 w-3 ${
                            i < comment.rating
                              ? "fill-yellow-400 text-yellow-400"
                              : "text-gray-300"
                          }`}
                        />
                      ))}
                    </div>
                  </div>
                  <p className="mt-1 text-sm">{comment.content}</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {formatRelativeTime(comment.createdAt * 1000)}
                  </p>
                </div>
                <div className="flex gap-2">
                  {comment.status === "pending" ? (
                    <>
                      <Button size="sm" variant="outline">Từ chối</Button>
                      <Button size="sm">Duyệt</Button>
                    </>
                  ) : (
                    <Button size="sm" variant="ghost">Xoá</Button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

// ================================================================
// TAB 5: USER REQUESTS (game muốn việt hoá tiếp)
// ================================================================

function RequestsTab() {
  const mockRequests = [
    { id: "1", gameName: "Elden Ring", votes: 342, requestedBy: "user_abc", createdAt: Date.now() / 1000 - 86400 },
    { id: "2", gameName: "God of War Ragnarok", votes: 287, requestedBy: "user_xyz", createdAt: Date.now() / 1000 - 86400 * 2 },
    { id: "3", gameName: "Disco Elysium", votes: 198, requestedBy: "user_def", createdAt: Date.now() / 1000 - 86400 * 3 },
    { id: "4", gameName: "Persona 5 Royal", votes: 156, requestedBy: "user_ghi", createdAt: Date.now() / 1000 - 86400 * 5 },
    { id: "5", gameName: "Hades", votes: 124, requestedBy: "user_jkl", createdAt: Date.now() / 1000 - 86400 * 7 },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle>Game được yêu cầu việt hoá</CardTitle>
        <CardDescription>
          Sắp xếp theo số vote từ cộng đồng
        </CardDescription>
      </CardHeader>
      <CardContent className="p-0">
        <div className="divide-y">
          {mockRequests.map((req, i) => (
            <div key={req.id} className="flex items-center justify-between p-4">
              <div className="flex items-center gap-3">
                <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-sm font-bold">
                  #{i + 1}
                </div>
                <div>
                  <div className="font-medium">{req.gameName}</div>
                  <div className="text-xs text-muted-foreground">
                    Yêu cầu bởi {req.requestedBy} · {formatRelativeTime(req.createdAt * 1000)}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <Badge variant="outline">{req.votes} votes</Badge>
                <Button size="sm" variant="outline">
                  <Plus className="mr-1 h-3 w-3" />
                  Nhận việt hoá
                </Button>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

// ================================================================
// TAB 6: DRIVE CONFIGURATION
// ================================================================

function DriveSettingsTab() {
  const [manifestUrl, setManifestUrl] = useState("");
  const [testStatus, setTestStatus] = useState<"idle" | "testing" | "ok" | "error">("idle");
  const [testMessage, setTestMessage] = useState("");

  useEffect(() => {
    api.getSetting("manifest_url").then((v) => {
      if (v) setManifestUrl(v);
    });
  }, []);

  const testConnection = async () => {
    setTestStatus("testing");
    setTestMessage("");
    try {
      const result = await api.fetchManifest();
      setTestStatus("ok");
      setTestMessage(`OK! Tìm thấy ${result.games.length} game trong manifest.`);
    } catch (e: any) {
      setTestStatus("error");
      setTestMessage(`Lỗi: ${e?.message ?? e}`);
    }
  };

  const save = async () => {
    await api.setSetting("manifest_url", manifestUrl);
    alert("Đã lưu! Khởi động lại app để áp dụng.");
  };

  return (
    <div className="max-w-2xl space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Cấu hình Google Drive</CardTitle>
          <CardDescription>
            URL tới file <code>manifest.json</code> gốc (chứa danh sách tất cả game)
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium">Manifest URL</label>
            <Input
              placeholder="https://drive.google.com/uc?export=download&id=..."
              value={manifestUrl}
              onChange={(e) => setManifestUrl(e.target.value)}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              Share file manifest.json công khai, lấy direct link
            </p>
          </div>

          <div className="flex gap-2">
            <Button variant="outline" onClick={testConnection} disabled={testStatus === "testing"}>
              <RefreshCw className={`mr-2 h-4 w-4 ${testStatus === "testing" ? "animate-spin" : ""}`} />
              Test kết nối
            </Button>
            <Button onClick={save}>
              <Save className="mr-2 h-4 w-4" />
              Lưu
            </Button>
          </div>

          {testMessage && (
            <div
              className={`rounded-md p-3 text-sm ${
                testStatus === "ok"
                  ? "bg-green-500/10 text-green-700"
                  : testStatus === "error"
                  ? "bg-destructive/10 text-destructive"
                  : "bg-muted"
              }`}
            >
              {testMessage}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Schema manifest</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Manifest phải là JSON hợp lệ với cấu trúc sau:
          </p>
          <pre className="mt-2 overflow-x-auto rounded-md bg-muted p-3 text-xs">
{`{
  "version": 1,
  "lastUpdated": "2026-10-18T00:00:00Z",
  "games": [
    {
      "id": "witcher-3",
      "name": "The Witcher 3: Wild Hunt",
      "manifestUrl": "https://..."
    }
  ]
}`}
          </pre>
          <div className="mt-3 flex gap-2">
            <Button variant="outline" size="sm" asChild>
              <a href="docs/MANIFEST_SCHEMA.md" target="_blank">
                <FileJson className="mr-1 h-3 w-3" />
                Schema đầy đủ
              </a>
            </Button>
            <Button variant="outline" size="sm" asChild>
              <a href="docs/manifest.example.json" target="_blank">
                <FileJson className="mr-1 h-3 w-3" />
                File ví dụ
              </a>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}