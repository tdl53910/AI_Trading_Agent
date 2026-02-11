cat > README.md << 'EOF'
# 🤖 AI Trading Agent - Survival Mode

> **"I gave an AI $50 and told it 'pay for yourself or you die'"**

An autonomous trading agent that turns $50 into profit or dies trying. Built in Rust for speed, using Claude API for reasoning, running on a $4.5/month VPS.

## 🚀 Features

- **Survival Mode**: Agent dies if balance hits $0
- **10-minute Cycles**: Every 10 minutes:
  - ✅ Scans 500-1000 markets
  - ✅ Builds fair value estimate with Claude API
  - ✅ Finds mispricing > 8%
  - ✅ Calculates position size (Kelly Criterion, max 6% bankroll)
  - ✅ Executes trades
  - ✅ Pays its own API bill from profits
- **News Integration**: Continuously scours news sources for market data
- **Test Mode Toggle**: Switch between real and fake money
- **Web Dashboard**: Real-time monitoring and control interface
- **Self-Sustaining**: Pays for its own API calls and VPS costs

## 📊 Performance Goal

Turn **$50** into **$2,980+** (as demonstrated in the original example)

## 🛠️ Quick Start

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env