# Serializability Analysis Report
Generated: 2025-06-30 18:12:48
Extras: ['--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|2.65|N/A|✅|
| `login_flow` |Error|Error|0.09|N/A|N/A|
| `shopping_cart` |Error|Error|0.18|N/A|N/A|
| `state_machine` |Error|Error|0.07|N/A|N/A|
| `BGP_routing` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.08|N/A|✅|
| `bank` |Error|Error|13.55|N/A|N/A|
| `bank_without_yields` |Error|Error|167.36|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.10|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.08|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.55|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.09|N/A|✅|
| `ex` |Serializable|Serializable|0.88|N/A|✅|
| `flag_non_ser` |Not serializable|Not serializable|0.94|✅|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.12|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|10.10|N/A|✅|
| `fred_arith_simplified_until_2` |Serializable|Serializable|78.67|N/A|✅|
| `fred_arith_tricky` |Not serializable|Not serializable|0.89|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.66|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.81|✅|N/A|
| `funny` |Not serializable|Not serializable|0.55|✅|N/A|
| `globals` |Not serializable|Not serializable|0.56|✅|N/A|
| `if_expr` |Serializable|Serializable|0.10|N/A|✅|
| `if_while` |Serializable|Serializable|0.11|N/A|✅|
| `incrdecr` |Serializable|Serializable|106.37|N/A|✅|
| `less_simple_ser` |Error|Error|0.11|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.10|N/A|✅|
| `modulo` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `modulo_nonser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `multiple_requests` |Serializable|Serializable|4.20|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.11|N/A|✅|
| `nested_while` |Serializable|Serializable|0.10|N/A|✅|
| `nondet` |Serializable|Serializable|0.62|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.63|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.59|✅|N/A|
| `nondet_impl2` |Serializable|Serializable|31.80|N/A|✅|
| `self_loop` |Serializable|Serializable|0.11|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.16|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.10|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.07|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.56|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.11|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.91|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.55|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.30|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |Serializable|Serializable|140.34|N/A|✅|
| `stateful_firewall` |Not serializable|Not serializable|11.37|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.61|N/A|N/A|
| `stop2` |Serializable|Serializable|28.31|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.69|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.69|N/A|N/A|
| `stop4` |Serializable|Serializable|50.44|N/A|✅|
| `stop4a` |Serializable|Serializable|116.78|N/A|✅|
| `tricky2` |Not serializable|Not serializable|5.43|✅|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|3.46|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|4.61|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.11|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.11|N/A|✅|

## Summary
- Serializable: 31 (valid proofs: 31, invalid: 0)
- Not serializable: 15 (valid traces: 15, invalid: 0)
- Timeouts: 11, Errors: 6, Total: 63
