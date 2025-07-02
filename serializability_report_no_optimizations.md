# Serializability Analysis Report - _No_Optimizations
Generated: 2025-07-02 01:22:11
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|4.55|✅|
| `login_flow` |Error|2.29|N/A|
| `shopping_cart` |Error|2.45|N/A|
| `state_machine` |Error|2.32|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |Error|42.24|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |Not serializable|36.37|N/A|
| `ex` |Serializable|2.73|❌|
| `flag_non_ser` |Not serializable|32.47|N/A|
| `flag_non_ser_turned_ser` |Serializable|2.40|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |Serializable|15.85|✅|
| `fred_arith_simplified_until_2` |Not serializable|34.38|N/A|
| `fred_arith_tricky` |Not serializable|24.92|✅|
| `fred_arith_tricky2` |Not serializable|4.26|✅|
| `fred_arith_tricky3` |Not serializable|5.52|✅|
| `funny` |Not serializable|3.51|✅|
| `if_while_with_req` |Serializable|3.00|✅|
| `incrdecr` |Not serializable|34.68|N/A|
| `less_simple_ser` |Error|3.32|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |Not serializable|47.33|✅|
| `multiple_requests_updated` |Serializable|8.44|✅|
| `nested_while` |Serializable|2.79|✅|
| `nondet` |Serializable|4.57|❌|
| `nondet2` |Not serializable|4.33|✅|
| `nondet_impl` |Not serializable|3.91|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.99|✅|
| `self_loop2` |Serializable|3.74|✅|
| `simple_nonser` |Not serializable|3.48|✅|
| `simple_nonser2` |Not serializable|2.83|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.61|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.38|❌|
| `simple_nonser3` |Not serializable|3.27|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|15.55|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|13.15|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|34.72|N/A|
| `stop2` |Serializable|40.87|✅|
| `stop3` |Not serializable|6.14|✅|
| `stop3a` |Not serializable|35.21|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|42.62|N/A|
| `tricky3` |Not serializable|49.66|N/A|
| `tricky3_ser` |Not serializable|59.43|N/A|

## Summary
- Serializable: 13 (valid proofs: 10, invalid: 3)
- Not serializable: 22 (valid traces: 13, invalid: 9)
- Timeouts: 10, Errors: 5, Total: 50
