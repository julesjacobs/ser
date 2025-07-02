# Serializability Analysis Report - _Only_Bidirectional
Generated: 2025-07-02 01:26:15
Extras: ['--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|6.22|✅|
| `login_flow` |Error|3.24|N/A|
| `shopping_cart` |Error|3.64|N/A|
| `state_machine` |Error|3.47|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |Not serializable|41.17|N/A|
| `ex` |Serializable|3.89|❌|
| `flag_non_ser` |Not serializable|3.84|✅|
| `flag_non_ser_turned_ser` |Serializable|3.06|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |Serializable|14.62|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky` |Not serializable|32.77|✅|
| `fred_arith_tricky2` |Not serializable|4.45|✅|
| `fred_arith_tricky3` |Not serializable|6.32|✅|
| `funny` |Not serializable|2.96|✅|
| `if_while_with_req` |Serializable|2.44|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |Error|3.55|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |Serializable|7.72|✅|
| `nested_while` |Serializable|3.08|✅|
| `nondet` |Serializable|3.74|✅|
| `nondet2` |Not serializable|4.26|✅|
| `nondet_impl` |Not serializable|3.31|✅|
| `nondet_impl2` |Serializable|44.02|✅|
| `self_loop` |Serializable|1.97|✅|
| `self_loop2` |Serializable|3.58|✅|
| `simple_nonser` |Not serializable|3.39|✅|
| `simple_nonser2` |Not serializable|2.57|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.27|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.03|❌|
| `simple_nonser3` |Not serializable|3.05|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|17.03|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|9.11|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|33.23|N/A|
| `stop2` |Serializable|29.51|✅|
| `stop3` |Not serializable|3.78|✅|
| `stop3a` |Not serializable|33.80|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|28.51|✅|
| `tricky3` |Not serializable|53.12|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 14 (valid proofs: 12, invalid: 2)
- Not serializable: 18 (valid traces: 14, invalid: 4)
- Timeouts: 14, Errors: 4, Total: 50
