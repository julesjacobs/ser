# Serializability Analysis Report
Generated: 2025-06-30 18:20:12
Extras: ['--without-generate-less', '--without-smart-kleene-order', '--without-bidirectional']

|Example|Orig|Proof|CPU(s)|Trace|Proof Cert|
|--|--|--|--|--|--|
| `data_flow` |Serializable|Serializable|3.80|N/A|✅|
| `login_flow` |Error|Error|0.08|N/A|N/A|
| `shopping_cart` |Error|Error|0.19|N/A|N/A|
| `state_machine` |Error|Error|0.11|N/A|N/A|
| `BGP_routing` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `arithmetic_with_comments` |Serializable|Serializable|0.09|N/A|✅|
| `bank` |Error|Error|14.35|N/A|N/A|
| `bank_without_yields` |Error|Error|135.97|N/A|N/A|
| `boolean_ops` |Serializable|Serializable|0.08|N/A|✅|
| `complex_expr` |Serializable|Serializable|0.09|N/A|✅|
| `complex_while_with_yields` |SMPT Timeout|SMPT Timeout|0.99|N/A|N/A|
| `equality_check` |Serializable|Serializable|0.08|N/A|✅|
| `ex` |Serializable|Serializable|0.93|N/A|✅|
| `flag_non_ser` |SMPT Timeout|SMPT Timeout|0.34|N/A|N/A|
| `flag_non_ser_turned_ser` |Serializable|Serializable|0.12|N/A|✅|
| `fred2_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `fred_arith_simplified_until_1` |Serializable|Serializable|22.96|N/A|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|SMPT Timeout|1.34|N/A|N/A|
| `fred_arith_tricky` |Not serializable|Not serializable|1.07|✅|N/A|
| `fred_arith_tricky2` |Not serializable|Not serializable|0.67|✅|N/A|
| `fred_arith_tricky3` |Not serializable|Not serializable|0.97|✅|N/A|
| `funny` |Not serializable|Not serializable|0.56|✅|N/A|
| `globals` |Not serializable|Not serializable|0.53|✅|N/A|
| `if_expr` |Serializable|Serializable|0.09|N/A|✅|
| `if_while` |Serializable|Serializable|0.10|N/A|✅|
| `incrdecr` |SMPT Timeout|SMPT Timeout|1.42|N/A|N/A|
| `less_simple_ser` |Error|Error|0.11|N/A|N/A|
| `mixed_expr` |Serializable|Serializable|0.09|N/A|✅|
| `modulo` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `modulo_nonser` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `multiple_requests` |Serializable|Serializable|8.20|N/A|✅|
| `multiple_vars` |Serializable|Serializable|0.11|N/A|✅|
| `nested_while` |Serializable|Serializable|0.11|N/A|✅|
| `nondet` |Serializable|Serializable|36.99|N/A|✅|
| `nondet2` |Not serializable|Not serializable|6.54|✅|N/A|
| `nondet_impl` |Not serializable|Not serializable|0.67|✅|N/A|
| `nondet_impl2` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `self_loop` |Serializable|Serializable|0.11|N/A|✅|
| `self_loop2` |Serializable|Serializable|0.17|N/A|✅|
| `seq_expr` |Serializable|Serializable|0.10|N/A|✅|
| `simple_assign` |Serializable|Serializable|0.09|N/A|✅|
| `simple_nonser` |Not serializable|Not serializable|1.04|✅|N/A|
| `simple_nonser2` |Not serializable|Not serializable|0.52|✅|N/A|
| `simple_nonser2_minus_yields_is_ser` |Serializable|Serializable|0.10|N/A|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|Serializable|0.96|N/A|✅|
| `simple_nonser3` |Not serializable|Not serializable|0.51|✅|N/A|
| `simple_ser` |Serializable|Serializable|0.10|N/A|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|Not serializable|2.02|✅|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stateful_firewall` |Not serializable|Not serializable|11.29|✅|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `stop` |SMPT Timeout|SMPT Timeout|0.62|N/A|N/A|
| `stop2` |Serializable|Serializable|65.68|N/A|✅|
| `stop3` |Not serializable|Not serializable|0.64|✅|N/A|
| `stop3a` |SMPT Timeout|SMPT Timeout|0.67|N/A|N/A|
| `stop4` |Serializable|Serializable|134.63|N/A|✅|
| `stop4a` |SMPT Timeout|SMPT Timeout|0.00|N/A|N/A|
| `tricky2` |SMPT Timeout|SMPT Timeout|4.46|N/A|N/A|
| `tricky3` |SMPT Timeout|SMPT Timeout|6.13|N/A|N/A|
| `tricky3_ser` |SMPT Timeout|SMPT Timeout|9.93|N/A|N/A|
| `while_expr` |Serializable|Serializable|0.10|N/A|✅|
| `yield_expr` |Serializable|Serializable|0.10|N/A|✅|

## Summary
- Serializable: 26 (valid proofs: 26, invalid: 0)
- Not serializable: 13 (valid traces: 13, invalid: 0)
- Timeouts: 18, Errors: 6, Total: 63
