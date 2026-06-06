import { createFileRoute } from "@tanstack/react-router";
import { GlossaryPage } from "@/features/glossary/glossary-page";

export const Route = createFileRoute("/glossary")({
  component: GlossaryPage,
});
