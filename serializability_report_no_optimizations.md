# Serializability Analysis Report - _No_Optimizations
Generated: 2025-07-02 00:12:41
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|4.78|✅|
| `login_flow` |Error|2.53|N/A|
| `shopping_cart` |Error|2.70|N/A|
| `state_machine` |Error|2.55|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|3.04|❌|
| `flag_non_ser` |Not serializable|7.72|N/A|
| `flag_non_ser_turned_ser` |Serializable|2.58|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |Not serializable|9.03|N/A|
| `fred_arith_tricky` |Not serializable|8.62|N/A|
| `fred_arith_tricky2` |Not serializable|3.95|✅|
| `fred_arith_tricky3` |Not serializable|5.48|✅|
| `funny` |Not serializable|2.99|✅|
| `if_while_with_req` |Serializable|2.90|✅|
| `incrdecr` |Not serializable|9.70|N/A|
| `less_simple_ser` |Error|3.68|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.42|✅|
| `nondet` |Serializable|5.46|❌|
| `nondet2` |Not serializable|5.10|✅|
| `nondet_impl` |Not serializable|3.30|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.41|✅|
| `self_loop2` |Serializable|3.91|✅|
| `simple_nonser` |Not serializable|3.18|✅|
| `simple_nonser2` |Not serializable|3.13|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.70|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.17|❌|
| `simple_nonser3` |Not serializable|2.96|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|8.99|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|5.63|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 10 (valid proofs: 7, invalid: 3)
- Not serializable: 14 (valid traces: 9, invalid: 5)
- Timeouts: 22, Errors: 4, Total: 50
