interface Task {
  id: number;
  is_active: boolean;
  execution_count: number;
  bounty_per_exec: number;
  remaining_funds: number;
}

interface TaskStatsProps {
  tasks: Task[];
}

export default function TaskStats({ tasks }: TaskStatsProps) {
  const activeTasks = tasks.filter((t) => t.is_active).length;
  const totalExecutions = tasks.reduce((sum, t) => sum + t.execution_count, 0);
  const totalBountyPaid = tasks.reduce(
    (sum, t) => sum + (t.bounty_per_exec * t.execution_count),
    0
  );
  const totalFunded = tasks.reduce((sum, t) => sum + t.remaining_funds, 0);

  const stats = [
    {
      label: 'Active Tasks',
      value: activeTasks,
      icon: '📋',
    },
    {
      label: 'Total Executions',
      value: totalExecutions,
      icon: '⚙️',
    },
    {
      label: 'Bounty Paid',
      value: `${totalBountyPaid} XLM`,
      icon: '💰',
    },
    {
      label: 'Funded Balance',
      value: `${totalFunded} XLM`,
      icon: '💵',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
      {stats.map((stat) => (
        <div key={stat.label} className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-gray-400 text-sm">{stat.label}</p>
              <p className="text-2xl font-bold text-white mt-2">{stat.value}</p>
            </div>
            <span className="text-4xl opacity-50">{stat.icon}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
