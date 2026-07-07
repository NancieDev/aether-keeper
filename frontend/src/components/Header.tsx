interface HeaderProps {
  pubKey: string | null;
  isConnecting: boolean;
  onConnect: () => Promise<void>;
  onDisconnect: () => void;
}

export default function Header({
  pubKey,
  isConnecting,
  onConnect,
  onDisconnect,
}: HeaderProps) {
  const truncateAddress = (addr: string) => {
    return `${addr.substring(0, 6)}...${addr.substring(addr.length - 6)}`;
  };

  return (
    <header className="border-b border-gray-700 bg-gray-900/50 backdrop-blur-sm sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-4 py-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center">
          <div className="flex items-center space-x-3">
            <div className="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-500 bg-clip-text text-transparent">
              🪐 Aether Keeper
            </div>
          </div>

          <button
            onClick={pubKey ? onDisconnect : onConnect}
            disabled={isConnecting}
            className="px-4 py-2 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 disabled:from-gray-600 disabled:to-gray-600 rounded-full font-medium text-white transition-all duration-200"
          >
            {isConnecting
              ? 'Connecting...'
              : pubKey
                ? truncateAddress(pubKey)
                : 'Connect Wallet'}
          </button>
        </div>
      </div>
    </header>
  );
}
