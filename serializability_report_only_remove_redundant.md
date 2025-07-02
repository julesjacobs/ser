# Serializability Analysis Report - _Only_Remove_Redundant
Generated: 2025-07-02 01:28:15
Extras: ['--without-bidirectional', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|7.42|✅|
| `login_flow` |Error|2.72|N/A|
| `shopping_cart` |Error|3.32|N/A|
| `state_machine` |Error|2.97|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |Not serializable|41.96|N/A|
| `ex` |Serializable|3.48|❌|
| `flag_non_ser` |Not serializable|33.31|N/A|
| `flag_non_ser_turned_ser` |Serializable|2.85|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |Serializable|28.46|✅|
| `fred_arith_simplified_until_2` |Not serializable|34.21|N/A|
| `fred_arith_tricky` |Not serializable|34.86|N/A|
| `fred_arith_tricky2` |Not serializable|4.94|✅|
| `fred_arith_tricky3` |Not serializable|7.63|✅|
| `funny` |Not serializable|2.72|✅|
| `if_while_with_req` |Serializable|2.20|✅|
| `incrdecr` |Not serializable|33.93|N/A|
| `less_simple_ser` |Error|3.28|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |Serializable|12.14|✅|
| `nested_while` |Serializable|3.16|✅|
| `nondet` |Serializable|5.74|❌|
| `nondet2` |Not serializable|4.93|✅|
| `nondet_impl` |Not serializable|4.04|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.74|✅|
| `self_loop2` |Serializable|4.55|✅|
| `simple_nonser` |Not serializable|3.48|✅|
| `simple_nonser2` |Not serializable|3.00|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.51|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.32|❌|
| `simple_nonser3` |Not serializable|3.38|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|29.62|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|13.61|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|36.53|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|5.01|✅|
| `stop3a` |Not serializable|35.21|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|46.30|N/A|
| `tricky3` |Not serializable|57.30|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 12 (valid proofs: 9, invalid: 3)
- Not serializable: 20 (valid traces: 11, invalid: 9)
- Timeouts: 14, Errors: 4, Total: 50
