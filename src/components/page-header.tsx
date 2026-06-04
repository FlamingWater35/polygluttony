interface PageHeaderProps {
  title: string;
  description?: string;
  actions?: React.ReactNode;
}

export function PageHeader({ title, description, actions }: PageHeaderProps) {
  return (
    <div className="flex items-start justify-between gap-4 border-b border-border px-5 py-3">
      <div className="flex flex-col gap-0.5">
        <h1 className="text-base font-semibold tracking-tight text-foreground">{title}</h1>
        {description ? (
          <p className="text-[12.5px] text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {actions}
    </div>
  );
}
