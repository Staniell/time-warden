import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function cleanAppName(name: string): string {
  return name.replace(/\.(exe|app|bat|cmd|sh)$/i, "");
}
