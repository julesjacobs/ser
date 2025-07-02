# Serializability Analysis Report - _Only_Bidirectional
Generated: 2025-07-02 00:13:41
Extras: ['--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|7.95|✅|
| `login_flow` |Error|2.77|N/A|
| `shopping_cart` |Error|3.84|N/A|
| `state_machine` |Error|3.22|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|3.85|❌|
| `flag_non_ser` |Not serializable|3.85|✅|
| `flag_non_ser_turned_ser` |Serializable|3.18|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky2` |Not serializable|6.34|✅|
| `fred_arith_tricky3` |SMPT Timeout|0.00|N/A|
| `funny` |Not serializable|3.39|✅|
| `if_while_with_req` |Serializable|2.49|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |Error|3.50|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.79|✅|
| `nondet` |Serializable|5.77|✅|
| `nondet2` |Not serializable|7.91|✅|
| `nondet_impl` |Not serializable|4.94|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.72|✅|
| `self_loop2` |Serializable|6.84|✅|
| `simple_nonser` |Not serializable|4.48|✅|
| `simple_nonser2` |Not serializable|4.00|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|3.34|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|4.80|❌|
| `simple_nonser3` |Not serializable|4.13|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |SMPT Timeout|0.00|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|7.52|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 10 (valid proofs: 8, invalid: 2)
- Not serializable: 9 (valid traces: 9, invalid: 0)
- Timeouts: 27, Errors: 4, Total: 50
