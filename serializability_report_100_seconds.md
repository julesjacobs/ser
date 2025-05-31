# Serializability Analysis Report

This report shows the serializability analysis results for all `.ser` examples using the new method with Petri net pruning.

## Results

| Example | Result | Description |
|---------|--------|-------------|
| `arithmetic` | ✅ Serializable | Arithmetic operations |
| `bank_without_yields` | ⚠️ Error | Analysis failed or timed out |
| `bank` | ❌ Not serializable | Banking transaction example |
| `BGP_routing` | ❌ Not serializable | BGP routing protocol |
| `boolean_ops` | ✅ Serializable | Boolean logic operations |
| `complex_expr` | ✅ Serializable | General example |
| `complex_while_with_yields` | ⚠️ Error | Analysis failed or timed out |
| `equality_check` | ✅ Serializable | General example |
| `flag_non_ser_turned_ser` | ✅ Serializable | Flag-based synchronization |
| `flag_non_ser` | ❌ Not serializable | Flag-based synchronization |
| `fred_arith_simplified_until_1` | ✅ Serializable | Fred example variant |
| `fred_arith_simplified_until_2` | ✅ Serializable | Fred example variant |
| `fred_arith` | ✅ Serializable | Fred example variant |
| `fred` | ✅ Serializable | Fred example variant |
| `fred2_arith` | ❌ Not serializable | Fred example variant |
| `fred2` | ❌ Not serializable | Fred example variant |
| `globals` | ❌ Not serializable | General example |
| `if_expr` | ✅ Serializable | Conditional constructs |
| `if_while` | ✅ Serializable | While loop constructs |
| `less_simple_ser` | ✅ Serializable | General example |
| `mixed_expr` | ✅ Serializable | General example |
| `multiple_requests` | ✅ Serializable | Multiple concurrent requests |
| `multiple_vars` | ✅ Serializable | General example |
| `nested_while` | ✅ Serializable | While loop constructs |
| `nondet_impl` | ❌ Not serializable | Non-deterministic behavior |
| `nondet_impl2` | ✅ Serializable | Non-deterministic behavior |
| `nondet` | ✅ Serializable | Non-deterministic behavior |
| `nondet2` | ❌ Not serializable | Non-deterministic behavior |
| `seq_expr` | ✅ Serializable | General example |
| `simple_assign` | ✅ Serializable | General example |
| `simple_nonser` | ❌ Not serializable | Basic non-serializable program |
| `simple_nonser2_minus_yields_is_ser` | ✅ Serializable | Basic non-serializable program |
| `simple_nonser2_turned_ser_with_locks` | ✅ Serializable | Basic non-serializable program |
| `simple_nonser3` | ❌ Not serializable | Basic non-serializable program |
| `simple_ser` | ✅ Serializable | Basic serializable program |
| `snapshot_isolation_network_monitoring_simplified` | ⚠️ Error | Analysis failed or timed out |
| `snapshot_isolation_network_monitoring` | ❌ Not serializable | Snapshot isolation example |
| `snapshot_isolation` | ❌ Not serializable | Snapshot isolation example |
| `stateful_firewall_without_yields` | ✅ Serializable | Stateful firewall example |
| `stateful_firewall` | ❌ Not serializable | Stateful firewall example |
| `while_expr` | ✅ Serializable | While loop constructs |
| `with_comments` | ✅ Serializable | General example |
| `yield_expr` | ✅ Serializable | Yield-based concurrency |

## Summary

- ✅ **Serializable**: Programs that maintain serializability properties
- ❌ **Not serializable**: Programs that violate serializability  
- ❓ **Unknown**: Could not determine result
- ⚠️ **Error**: Analysis failed or timed out

## Method

This analysis uses the new serializability checking method with Petri net pruning that:

1. Extracts zero variables from constraints using `extract_zero_variables`
2. Identifies nonzero variables (target places for filtering)  
3. Applies bidirectional iterative filtering to keep only relevant transitions
4. Uses SMPT (Satisfiability Modulo Petri Nets) for final reachability analysis

The pruning optimization removes transitions that cannot contribute to reaching nonzero places, potentially improving both performance and accuracy.

---

*Report generated automatically by analyze_examples.sh*
