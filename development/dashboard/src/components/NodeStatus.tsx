// Node status indicator component

interface NodeStatusProps {
  status: 'running' | 'stopped' | 'unknown';
}

export function NodeStatus({ status }: NodeStatusProps) {
  const statusConfig = {
    running: {
      color: 'bg-green-500',
      pulse: true,
    },
    stopped: {
      color: 'bg-red-500',
      pulse: false,
    },
    unknown: {
      color: 'bg-yellow-500',
      pulse: false,
    },
  };

  const config = statusConfig[status];

  return (
    <span
      className={`inline-block w-2.5 h-2.5 rounded-full ${config.color} ${config.pulse ? 'animate-pulse' : ''}`}
      title={status.charAt(0).toUpperCase() + status.slice(1)}
    />
  );
}
