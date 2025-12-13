# Serializability Analysis Report
Generated: 2025-06-30 17:48:50
Extras: []

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|2.66|N/A|✅|
| `login_flow` |Serializable|Serializable|20.30|N/A|✅|
| `shopping_cart` |Serializable|Serializable|110.95|N/A|✅|
| `state_machine` |Serializable|Serializable|22.03|N/A|✅|
| `BGP_routing` |Not serializable|Not serializable|4.95|✅|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.08|N/A|✅|
| `bank` |SMPT Timeout|SMPT Timeout|18.58|N/A|N/A|
| `bank_without_yields` |SMPT Timeout|SMPT Timeout|2.90|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.08|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.08|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.59|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.09|N/A|✅|
| `ex` |Serializable|Serializable|0.82|N/A|✅|
| `flag_non_ser` |Not serializable|Not serializable|0.94|✅|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.11|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.60|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|9.40|N/A|✅|
| `fred_arith_simplified_until_2` |Serializable|Serializable|73.82|N/A|✅|
| `fred_arith_tricky` |Not serializable|Not serializable|0.70|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.62|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.74|✅|N/A|
| `funny` |Not serializable|Not serializable|0.55|✅|N/A|
| `globals` |Not serializable|Not serializable|0.53|✅|N/A|
| `if_expr` |Serializable|Serializable|0.11|N/A|✅|
| `if_while` |Serializable|Serializable|0.11|N/A|✅|
| `incrdecr` |Serializable|Serializable|103.21|N/A|✅|
| `less_simple_ser` |Serializable|Serializable|201.71|N/A|✅|
| `mixed_expr` |Serializable|Serializable|0.09|N/A|✅|
| `modulo` |Serializable|Serializable|32.58|N/A|✅|
| `modulo_nonser` |Not serializable|Not serializable|0.70|✅|N/A|
| `multiple_requests` |Serializable|Serializable|4.15|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.09|N/A|✅|
| `nested_while` |Serializable|Serializable|0.09|N/A|✅|
| `nondet` |Serializable|Serializable|0.58|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.62|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.56|✅|N/A|
| `nondet_impl2` |Serializable|Serializable|30.75|N/A|✅|
| `self_loop` |Serializable|Serializable|0.10|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.18|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.12|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|0.96|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.54|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.82|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.52|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.32|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |Serializable|Serializable|129.36|N/A|✅|
| `stateful_firewall` |Not serializable|Not serializable|11.20|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.58|N/A|N/A|
| `stop2` |Serializable|Serializable|24.25|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.70|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.68|N/A|N/A|
| `stop4` |Serializable|Serializable|53.53|N/A|✅|
| `stop4a` |Serializable|Serializable|129.24|N/A|✅|
| `tricky2` |Not serializable|Not serializable|2.66|✅|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|1.87|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|4.45|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.10|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.10|N/A|✅|

## Summary
- Serializable: 36 (valid proofs: 36, invalid: 0)
- Not serializable: 17 (valid traces: 17, invalid: 0)
- Timeouts: 10, Errors: 0, Total: 63
