# Serializability Analysis Report - _Only_Remove_Redundant
Generated: 2025-07-02 00:14:11
Extras: ['--without-bidirectional', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |SMPT Timeout|0.00|N/A|
| `login_flow` |Error|3.70|N/A|
| `shopping_cart` |Error|4.72|N/A|
| `state_machine` |Error|3.70|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|4.77|❌|
| `flag_non_ser` |Not serializable|9.22|N/A|
| `flag_non_ser_turned_ser` |Serializable|3.42|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |Not serializable|9.64|N/A|
| `fred_arith_tricky` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky2` |Not serializable|6.93|✅|
| `fred_arith_tricky3` |SMPT Timeout|0.00|N/A|
| `funny` |Not serializable|2.73|✅|
| `if_while_with_req` |Serializable|2.38|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |Error|5.18|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.89|✅|
| `nondet` |Serializable|9.97|❌|
| `nondet2` |Not serializable|8.34|✅|
| `nondet_impl` |Not serializable|5.17|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.67|✅|
| `self_loop2` |Serializable|7.17|✅|
| `simple_nonser` |Not serializable|5.15|✅|
| `simple_nonser2` |Not serializable|4.22|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|3.93|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|4.75|❌|
| `simple_nonser3` |Not serializable|4.64|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |SMPT Timeout|0.00|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|8.51|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 9 (valid proofs: 6, invalid: 3)
- Not serializable: 10 (valid traces: 8, invalid: 2)
- Timeouts: 27, Errors: 4, Total: 50
