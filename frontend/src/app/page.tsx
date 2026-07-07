'use client';

import { useState, useEffect } from 'react';
import { requestAccess } from '@stellar/freighter-api';
import Header from '@/components/Header';
import TaskDashboard from '@/components/TaskDashboard';
import CreateTaskModal from '@/components/CreateTaskModal';
import { WalletStore, useWallet } from '@/store/wallet';

export default function Home() {
  const [isConnecting, setIsConnecting] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const { pubKey, connect, disconnect } = useWallet();

  const handleConnectWallet = async () => {
    setIsConnecting(true);
    try {
      await connect();
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-950 via-gray-900 to-gray-950">
      <Header
        pubKey={pubKey}
        isConnecting={isConnecting}
        onConnect={handleConnectWallet}
        onDisconnect={disconnect}
      />

      <main className="max-w-7xl mx-auto px-4 py-12 sm:px-6 lg:px-8">
        {pubKey ? (
          <>
            <div className="mb-8 flex justify-between items-center">
              <div>
                <h2 className="text-3xl font-bold text-white">Automation Tasks</h2>
                <p className="mt-2 text-gray-400">
                  Manage your decentralized automation tasks
                </p>
              </div>
              <button
                onClick={() => setShowCreateModal(true)}
                className="px-6 py-3 bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-700 hover:to-pink-700 rounded-lg font-semibold text-white transition-all duration-200 shadow-lg hover:shadow-xl"
              >
                + Create Task
              </button>
            </div>

            <TaskDashboard />

            {showCreateModal && (
              <CreateTaskModal onClose={() => setShowCreateModal(false)} />
            )}
          </>
        ) : (
          <div className="text-center py-20">
            <div className="inline-block bg-gray-800 rounded-2xl p-12 border border-gray-700">
              <h2 className="text-2xl font-bold text-white mb-4">
                Welcome to Aether Keeper
              </h2>
              <p className="text-gray-400 mb-8 max-w-md">
                Connect your Freighter wallet to create and manage automated smart contract tasks
              </p>
              <button
                onClick={handleConnectWallet}
                disabled={isConnecting}
                className="px-8 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 rounded-lg font-semibold text-white transition-all duration-200"
              >
                {isConnecting ? 'Connecting...' : 'Connect Freighter Wallet'}
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
