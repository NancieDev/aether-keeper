import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import '../styles/globals.css';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'Aether Keeper - Decentralized Automation Network',
  description: 'Execute automated smart contract tasks on Soroban with off-chain keepers',
  keywords: ['Soroban', 'Stellar', 'Web3', 'Automation', 'Smart Contracts'],
  authors: [{ name: 'Aether Team' }],
  viewport: {
    width: 'device-width',
    initialScale: 1,
    maximumScale: 1,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className={`${inter.className} bg-gray-950 text-gray-50`}>
        {children}
      </body>
    </html>
  );
}
