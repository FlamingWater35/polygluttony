import { useEffect, type ReactNode } from "react"
import { NavRail } from "@/components/nav-rail"
import { StatusBar } from "@/components/status-bar"
import { onBackendEvent } from "@/lib/ipc"
import { useTranslationRun } from "@/stores/translation-store"
import { useGlossaryRun } from "@/stores/glossary-store"
import type { RunEvent } from "@/types/generated/RunEvent"
import type { GlossaryEvent } from "@/types/generated/GlossaryEvent"

export function AppLayout({ children }: { children: ReactNode }) {
  const applyEvent = useTranslationRun((s) => s.applyEvent)
  const applyGlossaryEvent = useGlossaryRun((s) => s.applyEvent)

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
    <div className="grid h-screen grid-cols-[auto_1fr] grid-rows-[1fr_auto] bg-background text-foreground">
      <div className="row-span-2">
        <NavRail />
      </div>
      <main className="min-h-0 overflow-auto">{children}</main>
      <StatusBar />
    </div>
  )
}
