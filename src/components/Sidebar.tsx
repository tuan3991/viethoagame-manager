import { Link, useLocation } from "react-router-dom";
import { Library, Settings, ShieldCheck, Info } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/lib/store";

const navItems = [
  { path: "/library", icon: Library, label: "Thư viện" },
  { path: "/settings", icon: Settings, label: "Cài đặt" },
  { path: "/admin", icon: ShieldCheck, label: "Admin" },
  { path: "/about", icon: Info, label: "Giới thiệu" },
];

export function Sidebar() {
  const location = useLocation();
  const adminUnlocked = useAppStore((s) => s.adminUnlocked);

  return (
    <aside className="flex w-56 flex-col border-r bg-card">
      <div className="flex h-14 items-center border-b px-4">
        <h1 className="text-lg font-bold text-primary">Viethoagame</h1>
        <span className="ml-2 text-xs text-muted-foreground">Manager</span>
      </div>
      <nav className="flex-1 space-y-1 p-2">
        {navItems.map((item) => {
          // Ẩn admin nếu chưa unlock
          if (item.path === "/admin" && !adminUnlocked) return null;
          const Icon = item.icon;
          const isActive = location.pathname.startsWith(item.path);
          return (
            <Link
              key={item.path}
              to={item.path}
              className={cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              <Icon className="h-4 w-4" />
              {item.label}
            </Link>
          );
        })}
      </nav>
      <div className="border-t p-3 text-xs text-muted-foreground">
        v0.1.0 · 2026
      </div>
    </aside>
  );
}