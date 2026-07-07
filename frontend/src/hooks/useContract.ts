'use client';

import { useState, useCallback } from 'react';
import { useWallet } from '@/store/wallet';
import { SorobanRpcClient } from '@/utils/soroban';

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

export interface RegisterTaskParams {
  targetContract: string;
  function: string;
  interval: number;
  bountyPerExec: number;
  initialFunding: number;
}

export function useContract() {
  const { pubKey } = useWallet();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const rpcClient = new SorobanRpcClient(
    process.env.NEXT_PUBLIC_RPC_URL || 'https://soroban-testnet.stellar.org',
    process.env.NEXT_PUBLIC_AETHER_CONTRACT_ID || ''
  );

  /**
   * Fetch all tasks from the contract
   */
  const fetchTasks = useCallback(async (): Promise<Task[]> => {
    if (!pubKey) {
      throw new Error('Wallet not connected');
    }

    setIsLoading(true);
    setError(null);

    try {
      // TODO: Implement RPC call to fetch all tasks
      // This requires:
      // 1. Build contract query for all task IDs
      // 2. For each task_id, call get_task() via RPC
      // 3. Parse Soroban SDK responses into Task objects
      // 4. Filter out deleted/invalid tasks
      
      const tasks: Task[] = [];
      // Placeholder implementation
      return tasks;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to fetch tasks';
      setError(message);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [pubKey]);

  /**
   * Register a new task
   */
  const registerTask = useCallback(
    async (params: RegisterTaskParams): Promise<number> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        // Validate input
        if (params.interval < 60) {
          throw new Error('Interval must be at least 60 seconds');
        }
        if (params.bountyPerExec <= 0) {
          throw new Error('Bounty must be greater than 0');
        }
        if (params.initialFunding < params.bountyPerExec) {
          throw new Error('Initial funding must be at least equal to bounty');
        }

        // TODO: Implement transaction building
        // This requires:
        // 1. Use @stellar/stellar-sdk to build Soroban invoke_contract TX
        // 2. Set function: "register_task"
        // 3. Set args: [creator, target_contract, function, args, interval, bounty, funding, token]
        // 4. Sign with Freighter via signTransaction()
        // 5. Submit to Soroban RPC
        // 6. Poll for confirmation
        // 7. Extract task_id from contract events
        
        const taskId = 0; // Placeholder
        return taskId;
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to register task';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  /**
   * Pause a task
   */
  const pauseTask = useCallback(
    async (taskId: number): Promise<void> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        // TODO: Implement pause_task transaction
        // Similar to registerTask but call pause_task(task_id)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to pause task';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  /**
   * Resume a task
   */
  const resumeTask = useCallback(
    async (taskId: number): Promise<void> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        // TODO: Implement resume_task transaction
        // Similar to registerTask but call resume_task(task_id)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to resume task';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  /**
   * Deposit funds to a task
   */
  const depositFunds = useCallback(
    async (taskId: number, amount: number): Promise<void> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        if (amount <= 0) {
          throw new Error('Amount must be greater than 0');
        }

        // TODO: Implement deposit_funds transaction
        // Call deposit_funds(task_id, amount)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to deposit funds';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  /**
   * Withdraw funds from a task
   */
  const withdrawFunds = useCallback(
    async (taskId: number, amount: number): Promise<void> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        if (amount <= 0) {
          throw new Error('Amount must be greater than 0');
        }

        // TODO: Implement withdraw_funds transaction
        // Call withdraw_funds(task_id, amount)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to withdraw funds';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  /**
   * Cancel a task
   */
  const cancelTask = useCallback(
    async (taskId: number): Promise<void> => {
      if (!pubKey) {
        throw new Error('Wallet not connected');
      }

      setIsLoading(true);
      setError(null);

      try {
        // TODO: Implement cancel_task transaction
        // Call cancel_task(task_id)
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Failed to cancel task';
        setError(message);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [pubKey]
  );

  return {
    fetchTasks,
    registerTask,
    pauseTask,
    resumeTask,
    depositFunds,
    withdrawFunds,
    cancelTask,
    isLoading,
    error,
  };
}
