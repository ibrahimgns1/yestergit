# yestergit

yestergit is a CLI tool for developers who forget what they actually did during the day when the daily stand-up meeting starts. It tracks your local git commits across multiple repos and manual notes, then uses AI to turn those messy technical logs into a coherent update you can actually say out loud.

## Installation

First, clone the repository and navigate into it:

```bash
git clone https://github.com/ibrahimgns1/yestergit.git
cd yestergit
```

Then, install it globally via cargo:

```bash
cargo install --path .
```


## Quick Start

**1. Let it find your work:**
Scan a directory to find and track all git repositories.
```bash
yestergit scan --path ~/projects
```

**2. Configure your AI companion:**
Set up any OpenAI-compatible API (OpenAI, Gemini, Ollama, etc.).
```bash
yestergit config \
  --set-url "https://YOUR_MODEL_ENDPOINT_URL" \
  --set-model "your-model-name" \
  --set-key "your-api-key"
```

**3. Survive the Stand-up:**
Collect everything you've done today and get an AI-generated summary.
```bash
yestergit summarize
```

## Usage Examples

**What did I do today? (Default view):**
Shows all commits and notes from today (or since Friday if it's Monday).
```bash
yestergit
```

**Check the last 3 days:**
```bash
yestergit --days 3
```

**Add a manual note (for things git doesn't see):**
```bash
yestergit note "Dealt with burn-out and questions like: will AI replace my job?"
```

**Tracked repositories:**
```bash
yestergit list
```
