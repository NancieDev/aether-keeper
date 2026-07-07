import { formatDistanceToNow } from 'date-fns';

interface Task {
  id: number;
  creator: string;
  target_contract: string;
  function: string;
  interval: number;
  bounty_per_exec: number;
  remaining_funds: number;
  is_active: boolean;
  execution_count: number;
  last_executed_at: number;
}

interface TaskTableProps {
  tasks: Task[];
}

export default function TaskTable({ tasks }: TaskTableProps) {
  const truncate = (str: string) => {
    return `${str.substring(0, 6)}...${str.substring(str.length - 6)}`;
  };

  return (
    <div className="card overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-700">
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Task ID
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Status
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Function
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Bounty
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Remaining
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Executions
              </th>
              <th className="px-6 py-3 text-left text-sm font-semibold text-gray-300">
                Last Executed
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700">
            {tasks.map((task) => (
              <tr key={task.id} className="hover:bg-gray-700/50 transition-colors">
                <td className="px-6 py-4 text-sm font-mono text-purple-400">
                  #{task.id}
                </td>
                <td className="px-6 py-4 text-sm">
                  <span
                    className={`px-3 py-1 rounded-full text-xs font-semibold ${
                      task.is_active
                        ? 'bg-green-900/30 text-green-400'
                        : 'bg-gray-700/30 text-gray-400'
                    }`}
                  >
                    {task.is_active ? 'Active' : 'Paused'}
                  </span>
                </td>
                <td className="px-6 py-4 text-sm font-mono text-gray-300">
                  {task.function}
                </td>
                <td className="px-6 py-4 text-sm text-gray-300">
                  {task.bounty_per_exec} XLM
                </td>
                <td className="px-6 py-4 text-sm text-gray-300">
                  {task.remaining_funds} XLM
                </td>
                <td className="px-6 py-4 text-sm text-gray-300">
                  {task.execution_count}
                </td>
                <td className="px-6 py-4 text-sm text-gray-400">
                  {task.last_executed_at > 0
                    ? formatDistanceToNow(new Date(task.last_executed_at * 1000), {
                        addSuffix: true,
                      })
                    : 'Never'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
