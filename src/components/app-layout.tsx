import type { ReactNode } from "react";
import { NavRail } from "@/components/nav-rail";
import { StatusBar } from "@/components/status-bar";

export function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="grid h-screen grid-cols-[auto_1fr] grid-rows-[1fr_auto] bg-background text-foreground">
      <div className="row-span-2">
        <NavRail />
      </div>
      <main className="min-h-0 overflow-auto">{children}</main>
      <StatusBar />
    </div>
  );
}
