import { cn } from "@/lib/utils";

/**
 * Toast container - stub cho shadcn toaster
 * Tạm thời render container rỗng.
 * Sẽ tích hợp react-hot-toast khi cần.
 */
export function Toaster() {
  return <div id="toast-root" className={cn("fixed bottom-4 right-4 z-50 space-y-2")} />;
}