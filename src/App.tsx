import { Routes, Route, Navigate } from "react-router-dom";
import { Sidebar } from "@/components/Sidebar";
import { StatusBar } from "@/components/StatusBar";
import { Library } from "@/routes/library";
import { GameDetail } from "@/routes/game-detail";
import { Settings } from "@/routes/settings";
import { Admin } from "@/routes/admin";
import { About } from "@/routes/about";
import { Toaster } from "@/components/ui/toaster";

export default function App() {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <main className="flex flex-1 flex-col">
        <div className="flex-1 overflow-y-auto">
          <Routes>
            <Route path="/" element={<Navigate to="/library" replace />} />
            <Route path="/library" element={<Library />} />
            <Route path="/game/:id" element={<GameDetail />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/admin" element={<Admin />} />
            <Route path="/about" element={<About />} />
          </Routes>
        </div>
        <StatusBar />
      </main>
      <Toaster />
    </div>
  );
}