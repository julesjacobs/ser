# Serializability Analysis Report
Generated: 2025-06-30 18:49:56
Extras: ['--without-remove-redundant']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|2.67|N/A|✅|
| `login_flow` |Serializable|Serializable|22.13|N/A|✅|
| `shopping_cart` |Error|Error|0.09|N/A|N/A|
| `state_machine` |Serializable|Serializable|22.87|N/A|✅|
| `BGP_routing` |Not serializable|Not serializable|4.32|✅|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.09|N/A|✅|
| `bank` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `bank_without_yields` |SMPT Timeout|SMPT Timeout|6.16|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.10|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.10|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.56|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.10|N/A|✅|
| `ex` |Serializable|Serializable|0.79|N/A|✅|
| `flag_non_ser` |Not serializable|Not serializable|0.98|✅|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.13|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.59|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|9.57|N/A|✅|
| `fred_arith_simplified_until_2` |Serializable|Serializable|75.65|N/A|✅|
| `fred_arith_tricky` |SMPT Timeout|SMPT Timeout|0.40|N/A|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.57|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.69|✅|N/A|
| `funny` |Not serializable|Not serializable|0.53|✅|N/A|
| `globals` |Not serializable|Not serializable|0.51|✅|N/A|
| `if_expr` |Serializable|Serializable|0.11|N/A|✅|
| `if_while` |Serializable|Serializable|0.11|N/A|✅|
| `incrdecr` |Serializable|Serializable|105.42|N/A|✅|
| `less_simple_ser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.10|N/A|✅|
| `modulo` |Serializable|Serializable|46.05|N/A|✅|
| `modulo_nonser` |Not serializable|Not serializable|0.97|✅|N/A|
| `multiple_requests` |Serializable|Serializable|4.37|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.11|N/A|✅|
| `nested_while` |Serializable|Serializable|0.12|N/A|✅|
| `nondet` |Serializable|Serializable|0.65|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.66|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.60|✅|N/A|
| `nondet_impl2` |Serializable|Serializable|30.45|N/A|✅|
| `self_loop` |Serializable|Serializable|0.10|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.17|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.09|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.01|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.59|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.85|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.54|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|8.23|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |Serializable|Serializable|131.54|N/A|✅|
| `stateful_firewall` |Not serializable|Not serializable|11.62|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.63|N/A|N/A|
| `stop2` |Serializable|Serializable|25.42|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.67|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.62|N/A|N/A|
| `stop4` |Serializable|Serializable|55.83|N/A|✅|
| `stop4a` |Serializable|Serializable|158.25|N/A|✅|
| `tricky2` |Not serializable|Not serializable|2.56|✅|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|1.92|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|4.75|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.12|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.11|N/A|✅|

## Summary
- Serializable: 34 (valid proofs: 34, invalid: 0)
- Not serializable: 16 (valid traces: 16, invalid: 0)
- Timeouts: 12, Errors: 1, Total: 63
