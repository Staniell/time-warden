export interface Schedule {
  id?: number;
  name: string;
  start_time: string; // HH:MM:SS
  end_time: string; // HH:MM:SS
  days: string[]; // ["Mon", "Tue", ...]
  expected_apps: string[];
  check_interval_secs: number;
  grace_period_secs: number;
  enabled: boolean;
}

export const DAYS_OF_WEEK = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
