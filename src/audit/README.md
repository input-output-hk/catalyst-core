# Audit Tooling

Independent verification tools for Catalyst voting results. These tools allow anyone to audit and verify the integrity of voting outcomes without needing to trust centralized authorities.

## Quick Start

### 1. Download Fund State

Download historical fund state from [here](https://github.com/input-output-hk/catalyst-core) to replay and audit the voting event.

The official published results are found in **activevoteplans.json**.

**activevoteplans.json** = FINAL RESULTS.

### 2. Build the Audit Tool

```bash
cargo build --release -p audit
```

### 3. Run Offline Audit

#### Option A: Cross-reference with Official Results

Compare your audit results with published Catalyst tallies:

```bash
OFFICIAL_RESULTS=/tmp/activevoteplans.json 
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --official-results $OFFICIAL_RESULTS
```

#### Option B: Generate Encrypted Tally with Gamma Scaling

```bash
BLOCK0=/tmp/fund9-leader-1/artifacts/block0.bin
FRAGMENTS_STORAGE=/tmp/fund9-leader-1/persist/leader-1
GAMMA=0.5
PRECISION=5

./target/release/offline --fragments $FRAGMENTS_STORAGE --block0 $BLOCK0 --gamma $GAMMA --precision $PRECISION
```

### 4. Generated Files

The offline audit creates three critical files:

- **ledger_after_tally.json** - Decrypted ledger state after tally *(should match official results!)*
- **ledger_before_tally.json** - Encrypted ledger state before tally
- **decryption_shares.json** - Decryption shares for each proposal

---

## ➡️ **NEXT STEP: Verify Individual Proposals**

**🔍 [Complete the audit process - Verify specific proposal results →](src/tally/README.md)**

After generating the audit files above, the next crucial step is to **independently verify individual proposal results** using cryptographic proof. This step:

- ✅ **Validates each proposal's decryption** was performed correctly
- ✅ **Provides mathematical proof** of result integrity  
- ✅ **Requires no trust** in election officials or committee members
- ✅ **Can be run by anyone** using publicly available data

**Why this step is essential:**
- The offline audit gives you the raw encrypted data
- The tally verification proves the decryption was legitimate
- Together, they provide complete end-to-end verification

---

## Additional Tools

### Find Your Vote
[See instructions on how to find your voting history →](src/find/README.md)

### Regenerate Results from Live Node
If you want to regenerate **activevoteplans.json** yourself via a live node and historical fragments:
[See instructions here →](./balance/README.md)

## Overview

This audit tooling provides:

1. **Offline Verification** - Replay voting events from blockchain data
2. **Cryptographic Proof** - Mathematically verify decryption integrity
3. **Individual Vote Tracking** - Find and verify your specific votes
4. **Complete Transparency** - No black boxes or trusted components

The audit process is designed to be completely independent and reproducible, ensuring that anyone can verify Catalyst voting results without needing to trust any centralized authority.
```
