import type { LogEntry } from "../lib/types";
import usePlaidTokenExchange from "../hooks/usePlaidTokenExchange";

type Props = {
  token: string;
  setLogs: React.Dispatch<React.SetStateAction<LogEntry[]>>;
};

const PlaidLinkButton = ({ token, setLogs }: Props) => {
  const { open, ready, showButton } = usePlaidTokenExchange(token, setLogs);

  if (!showButton) return null;

  return (
    <button
      disabled={!ready}
      onClick={() => open()}
      className="w-[772px] rounded-sm border-[1px] border-bg-muted hover:bg-accent hover:text-bg hover:border-white transition-all cursor-pointer py-4  bg-accent/75 border-accent/50 font-mono text-white"
    >
      LINK_ACCOUNT &rarr;
    </button>
  );
};

export default PlaidLinkButton;
