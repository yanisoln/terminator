import React from "react";

interface LogMessage {
  id: number;
  timestamp: string;
  type: "info" | "error" | "success" | "sdk-out" | "sdk-err";
  message: string;
}

interface LogConsoleProps {
  logs: LogMessage[];
}

const LogConsole: React.FC<LogConsoleProps> = ({ logs }) => {
  return (
    <div className="mt-8">
      <h2 className="mb-4 text-xl font-semibold tracking-tight">Console</h2>
      <div className="h-64 overflow-y-auto rounded-md border bg-muted p-4 font-mono text-sm">
        {logs.length === 0 ? (
          <p className="text-muted-foreground">No logs yet. Trigger an SDK action.</p>
        ) : (
          logs.map((log) => (
            <div key={log.id} className={`mb-1 ${log.type === 'error' || log.type === 'sdk-err' ? 'text-red-500' : log.type === 'success' ? 'text-green-500' : ''}`}>
              <span className="text-muted-foreground mr-2">{log.timestamp}</span>
              <span>[{log.type.toUpperCase()}]</span>
              <span className="ml-2 whitespace-pre-wrap">{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default LogConsole;