# AI Trading Agent

An autonomous trading system built in Rust that evaluates markets, estimates fair value using an LLM, and executes trades under strict risk controls.

Designed to run on low-cost infrastructure and operate continuously with minimal manual intervention.

---

## Overview

This project implements a fully automated trading loop that:

- Scans hundreds of markets
- Uses an LLM to generate structured valuation reasoning
- Identifies statistically meaningful mispricing
- Sizes positions using bankroll-aware risk constraints
- Executes trades via broker integration
- Tracks performance and operating costs

The system is modular, testable, and designed for experimentation in small-capital environments.

---

## Core Architecture

Every cycle (default: 10 minutes), the agent:

1. Scans available markets
2. Generates structured fair value estimates
3. Filters for pricing dislocations
4. Applies risk management constraints
5. Executes trades
6. Logs performance metrics

Modules include:

- `agent/` – trading logic and orchestration
- `market/` – market scanning and data ingestion
- `news/` – event/news signal analysis
- `llm/` – model integration
- `risk` (within agent) – position sizing + bankroll protection
- `web/` – monitoring dashboard
- `tests/` – unit tests for trading logic

---

## Risk Controls

- Maximum position cap per trade
- Bankroll-aware sizing
- Mispricing threshold filtering
- Test mode toggle for simulation

This system is experimental and intended for research and controlled deployment.

---

## Deployment

Designed to run on:

- Rust backend
- Low-cost VPS
- API-based LLM reasoning
- Optional web dashboard for monitoring

---

## Quick Start

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
