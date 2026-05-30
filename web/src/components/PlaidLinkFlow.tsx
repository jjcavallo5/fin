import usePlaidAuthentication from "../hooks/usePlaidAuthentication";
import PlaidLinkButton from "./PlaidLinkButton";

const PlaidLinkFlow = () => {
  const { token, logs, setLogs } = usePlaidAuthentication();

  return (
    <>
      <div className="border-bg-muted border-[1px] rounded-sm px-4 py-6 w-[772px]">
        <p className="text-text-muted mb-2">Logs</p>
        {logs.map((log) => (
          <div className="flex flex-row gap-4 w-full">
            <p
              className={
                log.level === "error"
                  ? "text-red-500 w-[72px]"
                  : log.level === "info"
                    ? "text-blue-300 w-[72px]"
                    : "text-green-500 w-[72px]"
              }
            >
              {log.level.toUpperCase()}
            </p>
            <p className="text-text-muted">{log.time}</p>
            <p className="text-text">{log.message}</p>
          </div>
        ))}
      </div>

      {token && <PlaidLinkButton token={token} setLogs={setLogs} />}
    </>
  );
};

export default PlaidLinkFlow;
