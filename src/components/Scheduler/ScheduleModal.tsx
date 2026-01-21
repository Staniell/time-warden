import { useState, useEffect } from "react";
import { X, Plus, AlertCircle } from "lucide-react";
import { Schedule, DAYS_OF_WEEK } from "../../types";
import { clsx } from "clsx";

interface ScheduleModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (schedule: Schedule) => void;
  initialData?: Schedule;
}

const DEFAULT_SCHEDULE: Schedule = {
  name: "",
  start_time: "09:00",
  end_time: "17:00",
  days: ["Mon", "Tue", "Wed", "Thu", "Fri"],
  expected_apps: [],
  check_interval_secs: 5,
  grace_period_secs: 30,
  enabled: true,
};

export function ScheduleModal({ isOpen, onClose, onSave, initialData }: ScheduleModalProps) {
  const [formData, setFormData] = useState<Schedule>(DEFAULT_SCHEDULE);
  const [newApp, setNewApp] = useState("");

  useEffect(() => {
    if (isOpen) {
      setFormData(initialData || DEFAULT_SCHEDULE);
    }
  }, [isOpen, initialData]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Validate time format roughly (HH:MM is what input type="time" gives)
    // We append :00 for seconds if missing
    const finalData = {
      ...formData,
      start_time: formData.start_time.length === 5 ? `${formData.start_time}:00` : formData.start_time,
      end_time: formData.end_time.length === 5 ? `${formData.end_time}:00` : formData.end_time,
    };
    onSave(finalData);
  };

  const toggleDay = (day: string) => {
    setFormData((prev) => {
      const days = prev.days.includes(day) ? prev.days.filter((d) => d !== day) : [...prev.days, day];
      // Sort days based on week order
      return {
        ...prev,
        days: days.sort((a, b) => DAYS_OF_WEEK.indexOf(a) - DAYS_OF_WEEK.indexOf(b)),
      };
    });
  };

  const addApp = () => {
    if (newApp.trim()) {
      setFormData((prev) => ({
        ...prev,
        expected_apps: [...prev.expected_apps, newApp.trim()],
      }));
      setNewApp("");
    }
  };

  const removeApp = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      expected_apps: prev.expected_apps.filter((_, i) => i !== index),
    }));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-in fade-in duration-200">
      <div
        className="w-full max-w-lg bg-zinc-900 border border-zinc-800 rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <form onSubmit={handleSubmit}>
          <div className="flex items-center justify-between p-6 border-b border-zinc-800">
            <h2 className="text-xl font-bold text-white">{initialData ? "Edit Schedule" : "New Schedule"}</h2>
            <button
              type="button"
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          <div className="p-6 space-y-6 max-h-[70vh] overflow-y-auto">
            {/* Name */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-400">Schedule Name</label>
              <input
                type="text"
                required
                value={formData.name}
                onChange={(e) => setFormData((prev) => ({ ...prev, name: e.target.value }))}
                placeholder="e.g., Work Hours"
                className="w-full px-4 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all"
              />
            </div>

            {/* Time Range */}
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium text-zinc-400">Start Time</label>
                <input
                  type="time"
                  required
                  value={formData.start_time.slice(0, 5)}
                  onChange={(e) => setFormData((prev) => ({ ...prev, start_time: e.target.value }))}
                  className="w-full px-4 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium text-zinc-400">End Time</label>
                <input
                  type="time"
                  required
                  value={formData.end_time.slice(0, 5)}
                  onChange={(e) => setFormData((prev) => ({ ...prev, end_time: e.target.value }))}
                  className="w-full px-4 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                />
              </div>
            </div>

            {/* Days */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-400">Active Days</label>
              <div className="flex flex-wrap gap-2">
                {DAYS_OF_WEEK.map((day) => (
                  <button
                    key={day}
                    type="button"
                    onClick={() => toggleDay(day)}
                    className={clsx(
                      "w-10 h-10 rounded-lg text-sm font-medium transition-all",
                      formData.days.includes(day)
                        ? "bg-indigo-600 text-white shadow-lg shadow-indigo-500/20"
                        : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700 hover:text-white",
                    )}
                  >
                    {day.slice(0, 1)}
                  </button>
                ))}
              </div>
            </div>

            {/* Expected Apps */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-zinc-400">
                Expected Apps (Keywords)
                <span className="ml-2 text-xs text-zinc-500 font-normal">e.g., Code, Chrome, Slack</span>
              </label>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newApp}
                  onChange={(e) => setNewApp(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && (e.preventDefault(), addApp())}
                  placeholder="Add app title keyword..."
                  className="flex-1 px-4 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                />
                <button
                  type="button"
                  onClick={addApp}
                  className="px-3 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
                >
                  <Plus className="w-5 h-5" />
                </button>
              </div>
              <div className="flex flex-wrap gap-2 mt-2 min-h-8">
                {formData.expected_apps.map((app, i) => (
                  <span
                    key={i}
                    className="inline-flex items-center gap-1 pl-2 pr-1 py-1 rounded-md bg-indigo-500/10 border border-indigo-500/20 text-xs text-indigo-300"
                  >
                    {app}
                    <button
                      type="button"
                      onClick={() => removeApp(i)}
                      className="p-0.5 hover:bg-indigo-500/20 rounded-full transition-colors"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  </span>
                ))}
                {formData.expected_apps.length === 0 && (
                  <div className="text-xs text-amber-500/80 flex items-center gap-1.5 px-2 py-1 bg-amber-500/5 rounded-md border border-amber-500/10 w-full">
                    <AlertCircle className="w-3 h-3" />
                    No apps specified - all apps will be considered compliant.
                  </div>
                )}
              </div>
            </div>

            {/* Advanced Settings */}
            <div className="grid grid-cols-2 gap-4 pt-2 border-t border-zinc-800/50">
              <div className="space-y-2">
                <label className="text-xs font-medium text-zinc-500">Check Interval (s)</label>
                <input
                  type="number"
                  min="1"
                  max="60"
                  value={formData.check_interval_secs}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, check_interval_secs: parseInt(e.target.value) || 5 }))
                  }
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded-lg text-white text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
                />
              </div>
              <div className="space-y-2">
                <label className="text-xs font-medium text-zinc-500">Grace Period (s)</label>
                <input
                  type="number"
                  min="0"
                  max="300"
                  value={formData.grace_period_secs}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, grace_period_secs: parseInt(e.target.value) || 0 }))
                  }
                  className="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded-lg text-white text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
                />
              </div>
            </div>
          </div>

          <div className="p-6 border-t border-zinc-800 flex justify-end gap-3 bg-zinc-900">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-zinc-400 hover:text-white transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-6 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium shadow-lg shadow-indigo-500/20 transition-all"
            >
              {initialData ? "Save Changes" : "Create Schedule"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
