# Serializability Analysis Report - _Unoptimized
Generated: 2025-07-02 00:06:22
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-generate-less', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|7.60|✅|
| `login_flow` |Error|2.78|N/A|
| `shopping_cart` |Error|3.03|N/A|
| `state_machine` |Error|2.82|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|3.47|❌|
| `flag_non_ser` |Not serializable|7.97|N/A|
| `flag_non_ser_turned_ser` |Serializable|2.86|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |Not serializable|8.77|N/A|
| `fred_arith_tricky` |Not serializable|9.84|N/A|
| `fred_arith_tricky2` |Not serializable|4.69|✅|
| `fred_arith_tricky3` |Not serializable|8.04|✅|
| `funny` |Not serializable|2.42|✅|
| `if_while_with_req` |Serializable|2.06|✅|
| `incrdecr` |Not serializable|8.46|N/A|
| `less_simple_ser` |Error|2.82|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.50|✅|
| `nondet` |Serializable|7.20|❌|
| `nondet2` |Not serializable|6.12|✅|
| `nondet_impl` |Not serializable|4.33|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.44|✅|
| `self_loop2` |Serializable|4.76|✅|
| `simple_nonser` |Not serializable|3.51|✅|
| `simple_nonser2` |Not serializable|2.64|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.30|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.38|❌|
| `simple_nonser3` |Not serializable|2.75|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|9.70|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|5.65|✅|
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
