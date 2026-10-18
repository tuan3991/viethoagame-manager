import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export function Settings() {
  return (
    <div className="container mx-auto p-6">
      <h2 className="mb-6 text-3xl font-bold">Cài đặt</h2>
      <div className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle>Tổng quan</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <label className="text-sm font-medium">URL Manifest Drive</label>
              <Input placeholder="https://drive.google.com/.../manifest.json" />
              <p className="mt-1 text-xs text-muted-foreground">
                URL công khai tới file manifest.json trên Google Drive
              </p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Giao diện</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Tính năng theme (light/dark/system) đang được phát triển
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Quyền riêng tư</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Telemetry opt-in giúp cải thiện app - đang được phát triển
            </p>
          </CardContent>
        </Card>

        <Button variant="destructive">Reset về mặc định</Button>
      </div>
    </div>
  );
}