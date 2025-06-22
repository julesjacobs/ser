# Serializability Analysis Report

This report shows the serializability analysis results for all `.ser` examples using both original and proof-based methods, with and without optimizations.

**Analysis Configuration:**
- Parallel jobs: 14
- Timeout: none
- Generated: 2025-06-23 00:51:11

## Results

| Example | Opt Original | Opt Proof | No-Opt Original | No-Opt Proof | Opt CPU (s) | No-Opt CPU (s) | Trace Valid |
|---------|--------------|-----------|-----------------|--------------|-------------|----------------|-------------|
| `BGP_routing` | SMPT Timeout | SMPT Timeout | SMPT Timeout | SMPT Timeout | 48.70 | 35.86 | N/A |
| `arithmetic` | Serializable | Serializable | Serializable | Serializable | 2.35 | 3.50 | N/A |
| `boolean_ops` | Serializable | Serializable | Serializable | Serializable | 2.33 | 3.53 | N/A |
| `complex_expr` | Serializable | Serializable | Serializable | Serializable | 2.31 | 3.42 | N/A |
| `equality_check` | Serializable | Serializable | Serializable | Serializable | 2.31 | 3.42 | N/A |
| `ex` | Serializable | Serializable | Serializable | Serializable | 3.04 | 3.79 | N/A |
| `flag_non_ser` | Not serializable | Not serializable | SMPT Timeout | SMPT Timeout | 3.79 | 13.39 | ⚠️ Inconsistent |
| `flag_non_ser_turned_ser` | Serializable | Serializable | Serializable | Serializable | 2.42 | 3.30 | N/A |
| `fred` | Serializable | Serializable | SMPT Timeout | SMPT Timeout | 16.11 | 15.13 | N/A |
| `fred2` | SMPT Timeout | SMPT Timeout | SMPT Timeout | SMPT Timeout | 16.82 | 19.16 | N/A |
| `fred2_arith` | SMPT Timeout | SMPT Timeout | Not serializable | Not serializable | 16.24 | 9.68 | ❌ |
| `fred_arith` | Serializable | Serializable | SMPT Timeout | SMPT Timeout | 15.87 | 14.69 | N/A |
| `fred_arith_simplified_until_1` | Serializable | Serializable | Serializable | Serializable | 5.47 | 6.46 | N/A |
| `fred_arith_simplified_until_2` | Serializable | Serializable | SMPT Timeout | SMPT Timeout | 9.27 | 14.97 | N/A |
| `fred_arith_tricky` | SMPT Timeout | SMPT Timeout | Not serializable | Not serializable | 16.04 | 14.49 | ❌ |
| `fred_arith_tricky2` | Not serializable | Not serializable | Not serializable | Not serializable | 5.35 | 6.71 | ✅ |
| `fred_arith_tricky3` | Not serializable | Not serializable | Not serializable | Not serializable | 7.17 | 9.60 | ✅ |
| `funny` | Not serializable | Not serializable | Not serializable | Not serializable | 3.97 | 4.52 | ✅ |
| `globals` | Not serializable | Not serializable | Not serializable | Not serializable | 3.40 | 4.25 | ✅ |
| `if_expr` | Serializable | Serializable | Serializable | Serializable | 5.81 | 3.07 | N/A |
| `if_while` | Serializable | Serializable | Serializable | Serializable | 2.89 | 2.37 | N/A |
| `incrdecr` | Serializable | Serializable | SMPT Timeout | SMPT Timeout | 9.13 | 14.60 | N/A |
| `less_simple_ser` | Serializable | Serializable | Serializable | Serializable | 7.06 | 7.64 | N/A |
| `mixed_expr` | Serializable | Serializable | Serializable | Serializable | 3.05 | 3.89 | N/A |
| `modulo_nonser` | Not serializable | Not serializable | Not serializable | Not serializable | 6.56 | 4.65 | ✅ |
| `multiple_requests` | Serializable | Serializable | Serializable | Serializable | 4.28 | 3.65 | N/A |
| `multiple_vars` | Serializable | Serializable | Serializable | Serializable | 3.88 | 2.56 | N/A |
| `nested_while` | Serializable | Serializable | Serializable | Serializable | 3.27 | 2.57 | N/A |
| `nondet` | Serializable | Serializable | Serializable | Serializable | 3.87 | 3.89 | N/A |
| `nondet2` | Not serializable | Not serializable | Not serializable | Not serializable | 19.17 | 19.02 | ✅ |
| `nondet_impl` | Not serializable | Not serializable | Not serializable | Not serializable | 3.70 | 3.85 | ✅ |
| `nondet_impl2` | Serializable | Serializable | Serializable | Serializable | 5.73 | 4.99 | N/A |
| `self_loop` | Serializable | Serializable | Serializable | Serializable | 2.79 | 2.24 | N/A |
| `self_loop2` | Serializable | Serializable | Serializable | Serializable | 4.45 | 3.79 | N/A |
| `seq_expr` | Serializable | Serializable | Serializable | Serializable | 2.55 | 2.15 | N/A |
| `simple_assign` | Serializable | Serializable | Serializable | Serializable | 1.85 | 2.22 | N/A |
| `simple_nonser` | Not serializable | Not serializable | Not serializable | Not serializable | 4.21 | 4.02 | ✅ |
| `simple_nonser2` | Not serializable | Not serializable | Not serializable | Not serializable | 3.57 | 2.49 | ✅ |
| `simple_nonser2_minus_yields_is_ser` | Serializable | Serializable | Serializable | Serializable | 2.23 | 2.93 | N/A |
| `simple_nonser2_turned_ser_with_locks` | Serializable | Serializable | Serializable | Serializable | 2.56 | 2.79 | N/A |
| `simple_nonser3` | Not serializable | Not serializable | Not serializable | Not serializable | 4.15 | 3.86 | ✅ |
| `simple_ser` | Serializable | Serializable | Serializable | Serializable | 2.92 | 2.99 | N/A |
| `snapshot_isolation_network_monitoring` | Not serializable | Not serializable | Not serializable | Not serializable | 22.10 | 19.45 | ✅ |
| `stateful_firewall` | Not serializable | Not serializable | Not serializable | Not serializable | 17.43 | 14.12 | ✅ |
| `stateful_firewall_without_yields` | Serializable | Serializable | Serializable | Serializable | 15.40 | 13.90 | N/A |
| `stop` | Serializable | Serializable | Serializable | Serializable | 6.06 | 7.50 | N/A |
| `stop2` | Serializable | Serializable | Serializable | Serializable | 7.43 | 8.59 | N/A |
| `stop3` | Not serializable | Not serializable | Not serializable | Not serializable | 5.02 | 6.75 | ✅ |
| `stop3a` | Serializable | Serializable | Serializable | Serializable | 12.64 | 15.25 | N/A |
| `stop4` | Serializable | Serializable | Serializable | Serializable | 8.61 | 9.81 | N/A |
| `stop4a` | Serializable | Serializable | Serializable | Serializable | 14.08 | 12.03 | N/A |
| `tricky2` | SMPT Timeout | SMPT Timeout | SMPT Timeout | SMPT Timeout | 24.11 | 17.45 | N/A |
| `tricky3` | SMPT Timeout | SMPT Timeout | SMPT Timeout | SMPT Timeout | 25.68 | 23.33 | N/A |
| `tricky3_ser` | SMPT Timeout | SMPT Timeout | SMPT Timeout | SMPT Timeout | 39.85 | 29.44 | N/A |
| `while_expr` | Serializable | Serializable | Serializable | Serializable | 4.04 | 2.75 | N/A |
| `with_comments` | Serializable | Serializable | Serializable | Serializable | 3.26 | 2.39 | N/A |
| `yield_expr` | Serializable | Serializable | Serializable | Serializable | 2.07 | 3.29 | N/A |

## Summary

- ✅ **Serializable**: Programs that maintain serializability properties
- ❌ **Not serializable**: Programs that violate serializability
- ❓ **Unknown**: Could not determine result
- ⚠️ **Error**: Analysis failed
- ⏱️ **SMPT Timeout**: SMPT verification timed out
- ⚠️ **Inconsistent**: Results differ between optimized and non-optimized runs (serious issue)

**Trace Valid Column**:
- ✅ **Valid trace**: The counterexample trace was successfully validated against the NS definition
- ❌ **Invalid trace**: The counterexample trace failed validation (indicates a bug)
- **N/A**: Not applicable (serializable programs don't have counterexample traces)

**Note**: Each example is analyzed twice - once with optimizations (default) and once with `--without-bidirectional` flag. The table shows results for all four combinations: Optimized Original/Proof methods and Non-optimized Original/Proof methods. CPU times compare performance impact of optimizations.

---

*Report generated automatically by analyze_examples.py*
