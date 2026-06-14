import { useEffect, type ReactNode } from "react"
import { NavRail } from "@/components/nav-rail"
import { StatusBar } from "@/components/status-bar"
import { AtmosphereBackdrop } from "@/components/atmosphere-backdrop"
import { onBackendEvent } from "@/lib/ipc"
import { useTranslationRun } from "@/stores/translation-store"
import { useGlossaryRun } from "@/stores/glossary-store"
import type { RunEvent } from "@/types/generated/RunEvent"
import type { GlossaryEvent } from "@/types/generated/GlossaryEvent"

export function AppLayout({ children }: { children: ReactNode }) {
  const applyEvent = useTranslationRun((s) => s.applyEvent)
  const applyGlossaryEvent = useGlossaryRun((s) => s.applyEvent)
  const running = useTranslationRun((s) => s.running)
  const gBusy = useGlossaryRun((s) => s.busy)
  const intensity = running || gBusy ? 1 : 0

  useEffect(() => {
    const un = onBackendEvent<RunEvent>("translation://event", (e) => applyEvent(e.payload))
    return () => {
      un.then((f) => f())
    }
  }, [applyEvent])

  useEffect(() => {
    const un = onBackendEvent<GlossaryEvent>("glossary://event", (e) =>
      applyGlossaryEvent(e.payload),
    )
    return () => {
      un.then((f) => f())
    }
  }, [applyGlossaryEvent])

  return (
    <>
      <AtmosphereBackdrop intensity={intensity} />
      <div className="relative z-10 grid h-screen grid-cols-[auto_1fr] grid-rows-[1fr_auto] bg-transparent text-foreground">
        <div className="row-span-2">
          <NavRail />
        </div>
        <main className="min-h-0 overflow-auto [view-transition-name:app-main]">{children}</main>
        <StatusBar />
      </div>
    </>
  )
}
