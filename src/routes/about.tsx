import { useEffect, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api } from "@/lib/api";

export function About() {
  const [version, setVersion] = useState("");

  useEffect(() => {
    api.getAppVersion().then(setVersion).catch(() => setVersion("0.1.0"));
  }, []);

  return (
    <div className="container mx-auto p-6">
      <h2 className="mb-6 text-3xl font-bold">Giới thiệu</h2>
      <Card>
        <CardHeader>
          <CardTitle>Viethoagame Manager</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <p>Phiên bản: {version}</p>
          <p className="text-muted-foreground">
            App quản lý và cài đặt bản việt hóa game cho Windows.
          </p>
          <p className="text-muted-foreground">
            Được xây dựng bằng Tauri + React + TypeScript.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}