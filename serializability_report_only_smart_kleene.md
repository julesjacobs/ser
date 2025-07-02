# Serializability Analysis Report - _Only_Smart_Kleene
Generated: 2025-07-02 00:15:14
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-generate-less']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|9.68|✅|
| `login_flow` |Error|2.85|N/A|
| `shopping_cart` |Error|3.26|N/A|
| `state_machine` |Error|3.60|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|3.88|❌|
| `flag_non_ser` |Not serializable|8.60|N/A|
| `flag_non_ser_turned_ser` |Serializable|3.22|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |Not serializable|9.83|N/A|
| `fred_arith_tricky` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky2` |Not serializable|7.45|✅|
| `fred_arith_tricky3` |SMPT Timeout|0.00|N/A|
| `funny` |Not serializable|2.98|✅|
| `if_while_with_req` |Serializable|2.35|✅|
| `incrdecr` |Not serializable|9.61|N/A|
| `less_simple_ser` |Error|5.51|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.76|✅|
| `nondet` |Serializable|9.47|❌|
| `nondet2` |Not serializable|7.58|✅|
| `nondet_impl` |Not serializable|5.46|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|3.40|✅|
| `self_loop2` |Serializable|6.71|✅|
| `simple_nonser` |Not serializable|4.35|✅|
| `simple_nonser2` |Not serializable|3.90|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.67|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.98|❌|
| `simple_nonser3` |Not serializable|3.36|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |SMPT Timeout|0.00|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|7.51|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 10 (valid proofs: 7, invalid: 3)
- Not serializable: 11 (valid traces: 8, invalid: 3)
- Timeouts: 25, Errors: 4, Total: 50
