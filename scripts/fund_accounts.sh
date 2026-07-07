#!/bin/bash

set -e

echo "💰 Funding Stellar Testnet Accounts via Friendbot"
echo ""

ACCOUNTS=("$@")

if [ ${#ACCOUNTS[@]} -eq 0 ]; then
    echo "Usage: ./fund_accounts.sh <account1> <account2> ..."
    echo "Example: ./fund_accounts.sh GAAAA... GBBBB..."
    exit 1
fi

for account in "${ACCOUNTS[@]}"; do
    echo "Funding $account..."
    
    response=$(curl -s -X POST "https://friendbot.stellar.org?addr=$account")
    
    if echo "$response" | grep -q "account_id"; then
        echo "✅ Successfully funded $account"
    else
        echo "⚠️  Could not fund $account (may already be funded)"
    fi
    
    sleep 1
done

echo ""
echo "✅ Funding complete!"
