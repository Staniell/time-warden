import { Calendar, Clock, Edit2, Play, Pause, Trash2, AlertTriangle } from "lucide-react";
import { Schedule } from "../../types";
import { cleanAppName } from "../../lib/utils";
import { clsx } from "clsx";

interface ScheduleItemProps {
  schedule: Schedule;
  onEdit: (schedule: Schedule) => void;
  onDelete: (id: number) => void;
  onToggle: (id: number, enabled: boolean) => void;
}

export function ScheduleItem({ schedule, onEdit, onDelete, onToggle }: ScheduleItemProps) {
  return (
    <div
      className={clsx(
        "group relative p-5 rounded-xl border transition-all duration-300",
        schedule.enabled
          ? "bg-zinc-900/40 border-zinc-800/50 hover:border-indigo-500/30 hover:bg-zinc-900/60"
          : "bg-zinc-900/20 border-zinc-800/30 opacity-75 grayscale-[0.5]",
      )}
    >
      {/* Header / Main Info */}
      <div className="flex items-start justify-between mb-4">
        <div>
          <h3
            className={clsx(
              "font-semibold text-lg flex items-center gap-2",
              schedule.enabled ? "text-white" : "text-zinc-400",
            )}
          >
            {schedule.name}
            {!schedule.enabled && (
              <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-500">Disabled</span>
            )}
          </h3>
          <div className="flex items-center gap-3 text-sm text-zinc-400 mt-1">
            <div className="flex items-center gap-1.5">
              <Clock className="w-3.5 h-3.5 text-indigo-400" />
              <span>
                {schedule.start_time.slice(0, 5)} - {schedule.end_time.slice(0, 5)}
              </span>
            </div>
            <span className="w-1 h-1 rounded-full bg-zinc-700" />
            <div className="flex items-center gap-1.5">
              <Calendar className="w-3.5 h-3.5 text-indigo-400" />
              <span className="truncate max-w-[200px]">{schedule.days.join(", ")}</span>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={() => onToggle(schedule.id!, !schedule.enabled)}
            className="p-2 rounded-lg bg-zinc-800/50 hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors"
            title={schedule.enabled ? "Disable" : "Enable"}
          >
            {schedule.enabled ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
          </button>
          <button
            onClick={() => onEdit(schedule)}
            className="p-2 rounded-lg bg-zinc-800/50 hover:bg-zinc-700 text-zinc-400 hover:text-white transition-colors"
            title="Edit"
          >
            <Edit2 className="w-4 h-4" />
          </button>
          <button
            onClick={() => onDelete(schedule.id!)}
            className="p-2 rounded-lg bg-zinc-800/50 hover:bg-red-500/20 text-zinc-400 hover:text-red-400 transition-colors"
            title="Delete"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Constraints */}
      <div className="space-y-3">
        {/* Apps */}
        <div className="flex flex-wrap gap-2">
          {schedule.expected_apps.length > 0 ? (
            schedule.expected_apps.map((app, i) => (
              <span
                key={i}
                className="px-2 py-1 rounded-md bg-zinc-800/50 border border-zinc-700/50 text-xs text-zinc-300 font-mono"
              >
                {cleanAppName(app)}
              </span>
            ))
          ) : (
            <span className="px-2 py-1 rounded-md bg-zinc-800/30 border border-zinc-800 text-xs text-zinc-500 font-mono italic">
              Any app allowed
            </span>
          )}
        </div>

        {/* Settings Detail */}
        <div className="flex items-center gap-4 text-xs text-zinc-500 pt-2 border-t border-zinc-800/50">
          <div className="flex items-center gap-1.5">
            <AlertTriangle className="w-3 h-3" />
            <span>Grace: {schedule.grace_period_secs}s</span>
          </div>
          <span>•</span>
          <div>Check every {schedule.check_interval_secs}s</div>
        </div>
      </div>
    </div>
  );
}
