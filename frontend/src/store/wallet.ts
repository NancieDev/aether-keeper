import { create } from 'zustand';
import { requestAccess, signTransaction } from '@stellar/freighter-api';

interface WalletStore {
  pubKey: string | null;
  isConnected: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
  signTx: (tx: string) => Promise<string>;
}

export const useWallet = create<WalletStore>((set) => ({
  pubKey: null,
  isConnected: false,

  connect: async () => {
    try {
      const access = await requestAccess();
      set({
        pubKey: access.address || null,
        isConnected: !!access.address,
      });
    } catch (error) {
      console.error('Failed to connect to Freighter:', error);
      throw error;
    }
  },

  disconnect: () => {
    set({ pubKey: null, isConnected: false });
  },

  signTx: async (tx: string) => {
    try {
      const signed = await signTransaction(tx, {
        networkPassphrase: 'Test SDF Network ; September 2015',
      });
      return signed;
    } catch (error) {
      console.error('Failed to sign transaction:', error);
      throw error;
    }
  },
}));
