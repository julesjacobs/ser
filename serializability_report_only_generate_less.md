# Serializability Analysis Report - _Only_Generate_Less
Generated: 2025-07-02 01:30:15
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|9.00|✅|
| `login_flow` |Serializable|30.52|✅|
| `shopping_cart` |Error|4.25|N/A|
| `state_machine` |Serializable|36.21|✅|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |Not serializable|48.87|N/A|
| `complex_while_with_yields` |Not serializable|43.78|N/A|
| `ex` |Serializable|3.98|❌|
| `flag_non_ser` |Not serializable|33.50|N/A|
| `flag_non_ser_turned_ser` |Serializable|3.36|✅|
| `fred2_arith` |Not serializable|41.12|N/A|
| `fred_arith` |Not serializable|35.90|N/A|
| `fred_arith_simplified_until_1` |Serializable|32.48|✅|
| `fred_arith_simplified_until_2` |Not serializable|34.50|N/A|
| `fred_arith_tricky` |Not serializable|35.42|N/A|
| `fred_arith_tricky2` |Not serializable|5.42|✅|
| `fred_arith_tricky3` |Not serializable|7.60|✅|
| `funny` |Not serializable|2.79|✅|
| `if_while_with_req` |Serializable|2.22|✅|
| `incrdecr` |Not serializable|33.74|N/A|
| `less_simple_ser` |SMPT Timeout|0.00|N/A|
| `modulo` |Serializable|56.09|✅|
| `modulo_nonser` |Not serializable|7.18|✅|
| `multiple_requests_updated` |Serializable|14.23|✅|
| `nested_while` |Serializable|2.24|✅|
| `nondet` |Serializable|5.78|❌|
| `nondet2` |Not serializable|4.60|✅|
| `nondet_impl` |Not serializable|2.98|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|1.91|✅|
| `self_loop2` |Serializable|4.17|✅|
| `simple_nonser` |Not serializable|3.49|✅|
| `simple_nonser2` |Not serializable|2.65|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.19|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.01|❌|
| `simple_nonser3` |Not serializable|2.69|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|29.35|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|10.85|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|34.72|N/A|
| `stop2` |Serializable|49.00|✅|
| `stop3` |Not serializable|5.89|✅|
| `stop3a` |Not serializable|36.12|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|44.78|N/A|
| `tricky3` |Not serializable|56.59|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 16 (valid proofs: 13, invalid: 3)
- Not serializable: 24 (valid traces: 12, invalid: 12)
- Timeouts: 9, Errors: 1, Total: 50
