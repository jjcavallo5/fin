export type LogEntry = {
  level: "info" | "error" | "success";
  time: string;
  message: string;
};
