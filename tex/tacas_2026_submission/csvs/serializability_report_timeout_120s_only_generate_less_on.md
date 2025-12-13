# Serializability Analysis Report
Generated: 2025-06-30 18:27:12
Extras: ['--without-remove-redundant', '--without-smart-kleene-order', '--without-bidirectional']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|3.99|N/A|✅|
| `login_flow` |Serializable|Serializable|21.81|N/A|✅|
| `shopping_cart` |Error|Error|0.12|N/A|N/A|
| `state_machine` |Serializable|Serializable|28.33|N/A|✅|
| `BGP_routing` |SMPT Timeout|SMPT Timeout|7.39|N/A|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.09|N/A|✅|
| `bank` |SMPT Timeout|SMPT Timeout|24.34|N/A|N/A|
| `bank_without_yields` |SMPT Timeout|SMPT Timeout|7.45|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.10|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.11|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.98|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.10|N/A|✅|
| `ex` |Serializable|Serializable|1.05|N/A|✅|
| `flag_non_ser` |SMPT Timeout|SMPT Timeout|0.35|N/A|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.14|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|1.04|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|1.70|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|23.53|N/A|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|SMPT Timeout|1.30|N/A|N/A|
| `fred_arith_tricky` |Not serializable|Not serializable|0.92|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.67|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.86|✅|N/A|
| `funny` |Not serializable|Not serializable|0.54|✅|N/A|
| `globals` |Not serializable|Not serializable|0.58|✅|N/A|
| `if_expr` |Serializable|Serializable|0.11|N/A|✅|
| `if_while` |Serializable|Serializable|0.11|N/A|✅|
| `incrdecr` |SMPT Timeout|SMPT Timeout|1.33|N/A|N/A|
| `less_simple_ser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.09|N/A|✅|
| `modulo` |Serializable|Serializable|44.89|N/A|✅|
| `modulo_nonser` |Not serializable|Not serializable|0.96|✅|N/A|
| `multiple_requests` |Serializable|Serializable|7.82|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.11|N/A|✅|
| `nested_while` |Serializable|Serializable|0.11|N/A|✅|
| `nondet` |Serializable|Serializable|37.15|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.62|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.64|✅|N/A|
| `nondet_impl2` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `self_loop` |Serializable|Serializable|0.11|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.17|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.11|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.21|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.49|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.94|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.53|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.91|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stateful_firewall` |Not serializable|Not serializable|11.18|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.60|N/A|N/A|
| `stop2` |Serializable|Serializable|44.66|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.83|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.64|N/A|N/A|
| `stop4` |Serializable|Serializable|186.52|N/A|✅|
| `stop4a` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `tricky2` |SMPT Timeout|SMPT Timeout|2.70|N/A|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|4.00|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|9.92|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.10|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.10|N/A|✅|

## Summary
- Serializable: 29 (valid proofs: 29, invalid: 0)
- Not serializable: 14 (valid traces: 14, invalid: 0)
- Timeouts: 19, Errors: 1, Total: 63
