'use client';

import { useState } from 'react';
import { useContract } from '@/hooks/useContract';
import { isValidAddress } from '@/utils/soroban';

interface CreateTaskModalProps {
  onClose: () => void;
  onSuccess?: () => void;
}

export default function CreateTaskModal({ onClose, onSuccess }: CreateTaskModalProps) {
  const [formData, setFormData] = useState({
    targetContract: '',
    function: '',
    interval: '86400', // 1 day default
    bountyPerExec: '10',
    initialFunding: '100',
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const { registerTask } = useContract();

  const validateForm = (): boolean => {
    const errors: Record<string, string> = {};

    if (!formData.targetContract.trim()) {
      errors.targetContract = 'Contract address is required';
    } else if (!isValidAddress(formData.targetContract)) {
      errors.targetContract = 'Invalid Stellar address format';
    }

    if (!formData.function.trim()) {
      errors.function = 'Function name is required';
    } else if (!/^[a-z_][a-z0-9_]*$/i.test(formData.function)) {
      errors.function = 'Invalid function name (alphanumeric and underscore only)';
    }

    const interval = parseInt(formData.interval);
    if (isNaN(interval)) {
      errors.interval = 'Interval must be a number';
    } else if (interval < 60) {
      errors.interval = 'Interval must be at least 60 seconds';
    }

    const bounty = parseFloat(formData.bountyPerExec);
    if (isNaN(bounty)) {
      errors.bountyPerExec = 'Bounty must be a number';
    } else if (bounty <= 0) {
      errors.bountyPerExec = 'Bounty must be greater than 0';
    }

    const funding = parseFloat(formData.initialFunding);
    if (isNaN(funding)) {
      errors.initialFunding = 'Funding must be a number';
    } else if (funding <= 0) {
      errors.initialFunding = 'Funding must be greater than 0';
    } else if (funding < bounty) {
      errors.initialFunding = 'Funding must be at least equal to bounty';
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const taskId = await registerTask({
        targetContract: formData.targetContract,
        function: formData.function,
        interval: parseInt(formData.interval),
        bountyPerExec: parseFloat(formData.bountyPerExec),
        initialFunding: parseFloat(formData.initialFunding),
      });

      // Success
      onSuccess?.();
      onClose();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to create task';
      setError(message);
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    // Clear validation error for this field when user starts typing
    if (validationErrors[field]) {
      setValidationErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-xl border border-gray-700 max-w-md w-full mx-4 shadow-xl">
        <div className="p-6 border-b border-gray-700">
          <h2 className="text-2xl font-bold text-white">Create Automation Task</h2>
          <p className="text-sm text-gray-400 mt-1">
            Set up a recurring task to automate your smart contract functions
          </p>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="bg-red-900/20 border border-red-800 rounded-lg p-3 text-red-400 text-sm">
              <p className="font-semibold">Error</p>
              <p className="mt-1">{error}</p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Target Contract Address *
            </label>
            <input
              type="text"
              placeholder="C..."
              className={`w-full px-3 py-2 bg-gray-700 border rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors ${
                validationErrors.targetContract ? 'border-red-500' : 'border-gray-600'
              }`}
              value={formData.targetContract}
              onChange={(e) => handleInputChange('targetContract', e.target.value)}
              disabled={isLoading}
            />
            {validationErrors.targetContract && (
              <p className="text-red-400 text-xs mt-1">{validationErrors.targetContract}</p>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Function Name *
            </label>
            <input
              type="text"
              placeholder="e.g., compound_yield"
              className={`w-full px-3 py-2 bg-gray-700 border rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors ${
                validationErrors.function ? 'border-red-500' : 'border-gray-600'
              }`}
              value={formData.function}
              onChange={(e) => handleInputChange('function', e.target.value)}
              disabled={isLoading}
            />
            {validationErrors.function && (
              <p className="text-red-400 text-xs mt-1">{validationErrors.function}</p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Interval (seconds) *
              </label>
              <input
                type="number"
                placeholder="86400"
                className={`w-full px-3 py-2 bg-gray-700 border rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors ${
                  validationErrors.interval ? 'border-red-500' : 'border-gray-600'
                }`}
                value={formData.interval}
                onChange={(e) => handleInputChange('interval', e.target.value)}
                disabled={isLoading}
                min="60"
              />
              {validationErrors.interval && (
                <p className="text-red-400 text-xs mt-1">{validationErrors.interval}</p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Bounty per Exec (XLM) *
              </label>
              <input
                type="number"
                placeholder="10"
                className={`w-full px-3 py-2 bg-gray-700 border rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors ${
                  validationErrors.bountyPerExec ? 'border-red-500' : 'border-gray-600'
                }`}
                value={formData.bountyPerExec}
                onChange={(e) => handleInputChange('bountyPerExec', e.target.value)}
                disabled={isLoading}
                step="0.1"
                min="0"
              />
              {validationErrors.bountyPerExec && (
                <p className="text-red-400 text-xs mt-1">{validationErrors.bountyPerExec}</p>
              )}
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Initial Funding (XLM) *
            </label>
            <input
              type="number"
              placeholder="100"
              className={`w-full px-3 py-2 bg-gray-700 border rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors ${
                validationErrors.initialFunding ? 'border-red-500' : 'border-gray-600'
              }`}
              value={formData.initialFunding}
              onChange={(e) => handleInputChange('initialFunding', e.target.value)}
              disabled={isLoading}
              step="0.1"
              min="0"
            />
            {validationErrors.initialFunding && (
              <p className="text-red-400 text-xs mt-1">{validationErrors.initialFunding}</p>
            )}
            <p className="text-xs text-gray-400 mt-2">
              Must be at least equal to bounty amount
            </p>
          </div>

          <div className="flex gap-3 pt-6 border-t border-gray-700">
            <button
              type="button"
              onClick={onClose}
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:text-gray-500 text-gray-300 rounded-lg font-medium transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="flex-1 px-4 py-2 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 disabled:from-gray-700 disabled:to-gray-700 disabled:text-gray-500 text-white rounded-lg font-medium transition-colors"
            >
              {isLoading ? 'Creating...' : 'Create Task'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
