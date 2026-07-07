'use client';

import { useEffect, useState } from 'react';
import TaskTable from './TaskTable';
import TaskStats from './TaskStats';
import { useContract } from '@/hooks/useContract';

export interface Task {
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
  max_executions: number;
}

export default function TaskDashboard() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { fetchTasks: contractFetchTasks, isLoading: contractLoading } = useContract();

  useEffect(() => {
    fetchTasks();
    // Poll for updates every 30 seconds
    const interval = setInterval(fetchTasks, 30000);
    return () => clearInterval(interval);
  }, []);

  const fetchTasks = async () => {
    try {
      setLoading(true);
      const fetchedTasks = await contractFetchTasks();
      setTasks(fetchedTasks);
      setError(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch tasks';
      setError(message);
      console.error('Task fetch error:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading || contractLoading) {
    return (
      <div className="flex justify-center py-12">
        <div className="text-gray-400">Loading tasks...</div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <TaskStats tasks={tasks} />

      {error && (
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4 text-red-400 flex justify-between items-center">
          <div>
            <p className="font-semibold">Error loading tasks</p>
            <p className="text-sm mt-1">{error}</p>
          </div>
          <button
            onClick={fetchTasks}
            className="px-3 py-1 bg-red-700 hover:bg-red-600 rounded text-sm font-medium whitespace-nowrap ml-4"
          >
            Retry
          </button>
        </div>
      )}

      {tasks.length === 0 && !error ? (
        <div className="card text-center py-12">
          <p className="text-gray-400">
            No tasks yet. Create your first automation task to get started.
          </p>
        </div>
      ) : (
        <TaskTable tasks={tasks} onRefresh={fetchTasks} />
      )}
    </div>
  );
}
