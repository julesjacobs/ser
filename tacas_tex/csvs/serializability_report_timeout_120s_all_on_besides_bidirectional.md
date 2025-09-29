# Serializability Analysis Report
Generated: 2025-06-30 18:43:16
Extras: ['--without-bidirectional']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|3.68|N/A|✅|
| `login_flow` |Serializable|Serializable|19.03|N/A|✅|
| `shopping_cart` |Serializable|Serializable|138.39|N/A|✅|
| `state_machine` |Serializable|Serializable|26.18|N/A|✅|
| `BGP_routing` |SMPT Timeout|SMPT Timeout|7.60|N/A|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.09|N/A|✅|
| `bank` |SMPT Timeout|SMPT Timeout|20.66|N/A|N/A|
| `bank_without_yields` |SMPT Timeout|SMPT Timeout|3.99|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.10|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.09|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.98|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.11|N/A|✅|
| `ex` |Serializable|Serializable|1.00|N/A|✅|
| `flag_non_ser` |SMPT Timeout|SMPT Timeout|0.33|N/A|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.13|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.85|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|1.53|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|22.13|N/A|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|SMPT Timeout|1.25|N/A|N/A|
| `fred_arith_tricky` |Not serializable|Not serializable|0.85|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.65|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.85|✅|N/A|
| `funny` |Not serializable|Not serializable|0.54|✅|N/A|
| `globals` |Not serializable|Not serializable|0.53|✅|N/A|
| `if_expr` |Serializable|Serializable|0.09|N/A|✅|
| `if_while` |Serializable|Serializable|0.09|N/A|✅|
| `incrdecr` |SMPT Timeout|SMPT Timeout|1.32|N/A|N/A|
| `less_simple_ser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.11|N/A|✅|
| `modulo` |Serializable|Serializable|34.55|N/A|✅|
| `modulo_nonser` |Not serializable|Not serializable|0.75|✅|N/A|
| `multiple_requests` |Serializable|Serializable|7.82|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.10|N/A|✅|
| `nested_while` |Serializable|Serializable|0.11|N/A|✅|
| `nondet` |Serializable|Serializable|36.98|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.63|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.61|✅|N/A|
| `nondet_impl2` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `self_loop` |Serializable|Serializable|0.11|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.16|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.11|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|0.99|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.52|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.97|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.51|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.98|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stateful_firewall` |Not serializable|Not serializable|11.39|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.63|N/A|N/A|
| `stop2` |Serializable|Serializable|64.02|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.69|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.68|N/A|N/A|
| `stop4` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop4a` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `tricky2` |SMPT Timeout|SMPT Timeout|2.47|N/A|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|3.81|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|10.01|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.12|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.09|N/A|✅|

## Summary
- Serializable: 29 (valid proofs: 29, invalid: 0)
- Not serializable: 14 (valid traces: 14, invalid: 0)
- Timeouts: 20, Errors: 0, Total: 63
