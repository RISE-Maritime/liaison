# Getting Started with Memory Profiling

## Quick Start (5 minutes)

### 1. Navigate to benchmarks directory
```bash
cd /workspace/benchmarks
```

### 2. Check dependencies
```bash
make check-deps
```

Expected output:
```
✓ valgrind
✓ cargo
✓ python3
```

If any are missing:
```bash
make install-deps
```

### 3. Run your first profile
```bash
make quick-profile
```

This will:
- Build the project in release mode
- Profile the server and client
- Generate a summary report
- Take about 10-30 seconds

### 4. View results
```bash
make show-latest
```

### 5. Set as baseline
```bash
make baseline
```

## What to Do Next

### Daily Development Workflow

Before making changes:
```bash
make quick-profile
make baseline
```

After making changes:
```bash
make quick-profile
make compare
```

### Run Specific Tests

Test for memory leaks:
```bash
make test-leak
```

Run all memory tests:
```bash
make test
```

Profile with detailed Massif data:
```bash
./memory_profile.sh --detailed
```

### Generate Reports

Full analysis with charts:
```bash
make plot
```

Export to CSV:
```bash
make export-csv
```

### Interactive Setup

For a guided experience:
```bash
./quick_start.sh
```

## Common Commands

| Command | Description |
|---------|-------------|
| `make help` | Show all available commands |
| `make quick-profile` | Quick profiling (recommended for development) |
| `make profile` | Full comprehensive profiling |
| `make test` | Run memory tests |
| `make analyze` | Analyze latest results |
| `make compare` | Compare with baseline |
| `make baseline` | Set current as baseline |
| `make clean` | Clean old results |

## Understanding Results

After profiling, you'll see output like:

```
========================================
Memory Profile Summary
========================================

--- SERVER MEMORY ---
Peak memory:       52.34 MB
Final memory:      48.12 MB
Snapshots:            100

--- CLIENT MEMORY ---
Peak memory:      124.67 MB
Final memory:     112.34 MB
Snapshots:            150
========================================
```

**What to look for:**
- Peak memory should be stable across runs (< 5% variance)
- Final memory should return to near baseline after cleanup
- Compare "Current" vs "Baseline" when making optimizations

## Troubleshooting

**"valgrind: command not found"**
```bash
sudo apt-get install valgrind
```

**"No results found"**
- Run `make quick-profile` first

**"Permission denied"**
```bash
chmod +x *.sh *.py
```

## Learn More

- **README.md** - Complete documentation
- **EXAMPLES.md** - Practical examples and workflows
- **SETUP_SUMMARY.md** - Detailed setup information

## Need Help?

```bash
# Show script help
./memory_profile.sh --help

# Show Python script help
python3 analyze_memory.py --help

# Show make targets
make help
```

---

**Ready to start?** Run `./quick_start.sh` for an interactive guide!
