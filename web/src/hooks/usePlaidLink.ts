import { useEffect, useState } from "react";
import { z } from "zod";

const LinkTokenResponseSchema = z.object({
  link_token: z.string(),
});

type LogEntry = {
  level: "info" | "error" | "success";
  time: string;
  message: string;
};

const usePlaidAuthentication = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [token, setToken] = useState<string | undefined>();

  const onLoad = async () => {
    setLogs((prev) => [
      ...prev,
      {
        level: "info",
        time: new Date().toLocaleTimeString(),
        message: "creating token...",
      },
    ]);
    const resp = await fetch("/get-token");
    const body = await resp.json();
    if (!resp.ok) {
      setLogs((prev) => [
        ...prev,
        {
          level: "error",
          time: new Date().toLocaleTimeString(),
          message: "failed to create token",
        },
      ]);
    } else {
      const { link_token } = LinkTokenResponseSchema.parse(body);
      setToken(link_token);
      setLogs((prev) => [
        ...prev,
        {
          level: "success",
          time: new Date().toLocaleTimeString(),
          message: "token created",
        },
      ]);
    }
  };

  useEffect(() => {
    onLoad();
  }, []);

  return { logs, token };
};

export default usePlaidAuthentication;
