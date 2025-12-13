# Serializability Analysis Report
Generated: 2025-06-30 18:07:15
Extras: ['--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order', '--without-bidirectional']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|3.54|N/A|✅|
| `login_flow` |Error|Error|0.08|N/A|N/A|
| `shopping_cart` |Error|Error|0.16|N/A|N/A|
| `state_machine` |Error|Error|0.07|N/A|N/A|
| `BGP_routing` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.09|N/A|✅|
| `bank` |Error|Error|13.20|N/A|N/A|
| `bank_without_yields` |Error|Error|189.19|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.08|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.08|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.93|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.08|N/A|✅|
| `ex` |Serializable|Serializable|0.85|N/A|✅|
| `flag_non_ser` |SMPT Timeout|SMPT Timeout|0.25|N/A|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.11|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|22.38|N/A|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|SMPT Timeout|1.17|N/A|N/A|
| `fred_arith_tricky` |Not serializable|Not serializable|0.93|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.60|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.83|✅|N/A|
| `funny` |Not serializable|Not serializable|0.54|✅|N/A|
| `globals` |Not serializable|Not serializable|0.45|✅|N/A|
| `if_expr` |Serializable|Serializable|0.09|N/A|✅|
| `if_while` |Serializable|Serializable|0.11|N/A|✅|
| `incrdecr` |SMPT Timeout|SMPT Timeout|1.27|N/A|N/A|
| `less_simple_ser` |Error|Error|0.11|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.10|N/A|✅|
| `modulo` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `modulo_nonser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `multiple_requests` |Serializable|Serializable|7.63|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.10|N/A|✅|
| `nested_while` |Serializable|Serializable|0.10|N/A|✅|
| `nondet` |Serializable|Serializable|36.83|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.52|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.65|✅|N/A|
| `nondet_impl2` |Serializable|Serializable|202.77|N/A|✅|
| `self_loop` |Serializable|Serializable|0.10|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.14|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.11|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.09|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.03|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.55|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.98|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.54|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.87|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stateful_firewall` |Not serializable|Not serializable|10.53|✅|N/A|
| `stateful_firewall_without_yields` |Serializable|Serializable|223.70|N/A|✅|
| `stop` |SMPT Timeout|SMPT Timeout|0.62|N/A|N/A|
| `stop2` |Serializable|Serializable|64.89|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.86|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.69|N/A|N/A|
| `stop4` |Serializable|Serializable|132.35|N/A|✅|
| `stop4a` |Serializable|Serializable|232.33|N/A|✅|
| `tricky2` |Not serializable|Not serializable|7.42|✅|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|6.06|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|9.71|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.09|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.11|N/A|✅|

## Summary
- Serializable: 29 (valid proofs: 29, invalid: 0)
- Not serializable: 14 (valid traces: 14, invalid: 0)
- Timeouts: 14, Errors: 6, Total: 63
