import { cn, cleanAppName } from "../lib/utils";
import { Clock } from "lucide-react";

// Matches the Rust Session struct
interface Session {
  id: number;
  app_id: string;
  app_name?: string;
  start_time: string;
  end_time?: string;
  duration_seconds?: number;
  is_idle: boolean;
}

interface ActivityListProps {
  sessions: Session[];
  className?: string;
}

export function ActivityList({ sessions, className }: ActivityListProps) {
  if (sessions.length === 0) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center p-8 text-zinc-500 bg-zinc-900/30 rounded-xl border border-dashed border-zinc-800",
          className,
        )}
      >
        <Clock className="w-8 h-8 mb-2 opacity-50" />
        <p className="text-sm">No activity recorded today</p>
      </div>
    );
  }

  return (
    <div
      className={cn(
        "space-y-1 bg-zinc-900/30 rounded-xl p-2 border border-zinc-800/50 max-h-[300px] overflow-y-auto pr-2 custom-scrollbar",
        className,
      )}
    >
      {sessions.map((session) => (
        <div
          key={session.id}
          className="flex items-center justify-between p-3 rounded-lg hover:bg-zinc-800/50 transition-colors group"
        >
          <div className="flex items-center gap-3">
            <div className={cn("w-2 h-2 rounded-full", session.is_idle ? "bg-amber-500/50" : "bg-indigo-500/50")} />
            <div>
              <div className="text-sm font-medium text-zinc-200">
                {cleanAppName(session.app_name || session.app_id)}
              </div>
              <div className="text-xs text-zinc-500 font-mono">
                {new Date(session.start_time).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
              </div>
            </div>
          </div>
          <div className="text-sm font-medium text-zinc-400 font-mono group-hover:text-zinc-200 transition-colors">
            {formatDuration(session.duration_seconds || 0)}
          </div>
        </div>
      ))}
    </div>
  );
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;

  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}
