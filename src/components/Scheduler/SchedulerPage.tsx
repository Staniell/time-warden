import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, Loader2 } from "lucide-react";
import { Schedule } from "../../types";
import { ScheduleItem } from "./ScheduleItem";
import { ScheduleModal } from "./ScheduleModal";

export function SchedulerPage() {
  const [schedules, setSchedules] = useState<Schedule[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingSchedule, setEditingSchedule] = useState<Schedule | undefined>(undefined);

  const fetchSchedules = async () => {
    try {
      setIsLoading(true);
      const data = await invoke<Schedule[]>("get_all_schedules");
      setSchedules(data);
    } catch (error) {
      console.error("Failed to fetch schedules:", error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchSchedules();
  }, []);

  const handleCreate = () => {
    setEditingSchedule(undefined);
    setIsModalOpen(true);
  };

  const handleEdit = (schedule: Schedule) => {
    setEditingSchedule(schedule);
    setIsModalOpen(true);
  };

  const handleDelete = async (id: number) => {
    if (confirm("Are you sure you want to delete this schedule?")) {
      try {
        await invoke("delete_schedule", { id });
        setSchedules((prev) => prev.filter((s) => s.id !== id));
      } catch (error) {
        console.error("Failed to delete schedule:", error);
      }
    }
  };

  const handleToggle = async (id: number, enabled: boolean) => {
    try {
      await invoke("toggle_schedule", { id, enabled });
      setSchedules((prev) => prev.map((s) => (s.id === id ? { ...s, enabled } : s)));
    } catch (error) {
      console.error("Failed to toggle schedule:", error);
    }
  };

  const handleSave = async (schedule: Schedule) => {
    try {
      if (schedule.id) {
        await invoke("update_schedule", { schedule });
      } else {
        await invoke("create_schedule", { schedule });
      }
      setIsModalOpen(false);
      fetchSchedules(); // Refresh list to get new IDs/updates
    } catch (error) {
      console.error("Failed to save schedule:", error);
      alert("Failed to save schedule. Check console for details.");
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold mb-2">Scheduler</h1>
          <p className="text-zinc-400">Manage your blocking schedules</p>
        </div>

        <button
          onClick={handleCreate}
          className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-colors font-medium shadow-lg shadow-indigo-500/20"
        >
          <Plus className="w-4 h-4" />
          Add Schedule
        </button>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-20">
          <Loader2 className="w-8 h-8 text-indigo-500 animate-spin" />
        </div>
      ) : schedules.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {schedules.map((schedule) => (
            <ScheduleItem
              key={schedule.id}
              schedule={schedule}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onToggle={handleToggle}
            />
          ))}
        </div>
      ) : (
        <div className="bg-zinc-900/40 rounded-2xl p-12 border border-zinc-800/50 backdrop-blur-sm flex flex-col items-center justify-center text-center">
          <div className="w-16 h-16 bg-zinc-800/50 rounded-full flex items-center justify-center mb-4">
            <Plus className="w-8 h-8 text-zinc-500" />
          </div>
          <h3 className="text-xl font-semibold mb-2">No schedules yet</h3>
          <p className="text-zinc-400 max-w-sm mb-6">
            Create a schedule to define when specific applications should be blocked or monitored.
          </p>
          <button onClick={handleCreate} className="text-indigo-400 hover:text-indigo-300 font-medium">
            Create your first schedule &rarr;
          </button>
        </div>
      )}

      <ScheduleModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSave={handleSave}
        initialData={editingSchedule}
      />
    </div>
  );
}
