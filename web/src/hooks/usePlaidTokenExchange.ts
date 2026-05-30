import { usePlaidLink } from "react-plaid-link";
import type { LogEntry } from "../lib/types";
import { useState } from "react";

const usePlaidTokenExchange = (
  token: string,
  setLogs: React.Dispatch<React.SetStateAction<LogEntry[]>>,
) => {
  const [showButton, setShowButton] = useState(true);

  const onSuccess = async (public_token: string, _metadata: any) => {
    setShowButton(false);
    setLogs((prev) => [
      ...prev,
      {
        level: "info",
        time: new Date().toLocaleTimeString(),
        message: "exchanging token...",
      },
    ]);
    const resp = await fetch("/exchange-token", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ public_token }),
    });

    if (!resp.ok) {
      setLogs((prev) => [
        ...prev,
        {
          level: "error",
          time: new Date().toLocaleTimeString(),
          message: "token exchange failed!",
        },
      ]);
    } else {
      setLogs((prev) => [
        ...prev,
        {
          level: "success",
          time: new Date().toLocaleTimeString(),
          message: "token exchange successful. You may now close this page.",
        },
      ]);
    }
  };

  const { open, ready } = usePlaidLink({ token, onSuccess });

  return { open, ready, showButton };
};

export default usePlaidTokenExchange;
