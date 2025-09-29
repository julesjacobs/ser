# Serializability Analysis Report
Generated: 2025-06-30 19:06:25
Extras: ['--without-smart-kleene-order']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|2.77|N/A|✅|
| `login_flow` |Serializable|Serializable|19.75|N/A|✅|
| `shopping_cart` |Serializable|Serializable|107.92|N/A|✅|
| `state_machine` |Serializable|Serializable|21.94|N/A|✅|
| `BGP_routing` |Not serializable|Not serializable|4.98|✅|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.10|N/A|✅|
| `bank` |SMPT Timeout|SMPT Timeout|18.97|N/A|N/A|
| `bank_without_yields` |SMPT Timeout|SMPT Timeout|2.96|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.11|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.10|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.56|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.08|N/A|✅|
| `ex` |Serializable|Serializable|0.89|N/A|✅|
| `flag_non_ser` |Not serializable|Not serializable|0.96|✅|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.14|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.64|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|9.26|N/A|✅|
| `fred_arith_simplified_until_2` |Serializable|Serializable|76.79|N/A|✅|
| `fred_arith_tricky` |Not serializable|Not serializable|30.39|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.88|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.73|✅|N/A|
| `funny` |Not serializable|Not serializable|0.54|✅|N/A|
| `globals` |Not serializable|Not serializable|0.54|✅|N/A|
| `if_expr` |Serializable|Serializable|0.10|N/A|✅|
| `if_while` |Serializable|Serializable|0.12|N/A|✅|
| `incrdecr` |Serializable|Serializable|104.26|N/A|✅|
| `less_simple_ser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.10|N/A|✅|
| `modulo` |Serializable|Serializable|34.07|N/A|✅|
| `modulo_nonser` |Not serializable|Not serializable|0.74|✅|N/A|
| `multiple_requests` |Serializable|Serializable|4.17|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.11|N/A|✅|
| `nested_while` |Serializable|Serializable|0.11|N/A|✅|
| `nondet` |Serializable|Serializable|0.61|N/A|✅|
| `nondet2` |Not serializable|Not serializable|0.59|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.57|✅|N/A|
| `nondet_impl2` |Serializable|Serializable|31.66|N/A|✅|
| `self_loop` |Serializable|Serializable|0.12|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.18|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.10|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.00|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.54|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.89|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.56|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|1.33|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stateful_firewall` |Not serializable|Not serializable|11.39|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.64|N/A|N/A|
| `stop2` |Serializable|Serializable|25.23|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.82|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.64|N/A|N/A|
| `stop4` |Serializable|Serializable|54.30|N/A|✅|
| `stop4a` |Serializable|Serializable|153.51|N/A|✅|
| `tricky2` |Not serializable|Not serializable|2.63|✅|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|2.01|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|4.91|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.09|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.10|N/A|✅|

## Summary
- Serializable: 34 (valid proofs: 34, invalid: 0)
- Not serializable: 17 (valid traces: 17, invalid: 0)
- Timeouts: 12, Errors: 0, Total: 63
