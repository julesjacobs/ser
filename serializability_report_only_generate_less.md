# Serializability Analysis Report - _Only_Generate_Less
Generated: 2025-07-02 00:14:44
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-smart-kleene-order']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |SMPT Timeout|0.00|N/A|
| `login_flow` |SMPT Timeout|0.00|N/A|
| `shopping_cart` |Error|5.33|N/A|
| `state_machine` |SMPT Timeout|0.00|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|5.70|❌|
| `flag_non_ser` |Not serializable|9.45|N/A|
| `flag_non_ser_turned_ser` |Serializable|4.40|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |Not serializable|9.93|N/A|
| `fred_arith_tricky` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky2` |Not serializable|7.94|✅|
| `fred_arith_tricky3` |SMPT Timeout|0.00|N/A|
| `funny` |Not serializable|4.39|✅|
| `if_while_with_req` |Serializable|3.21|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |SMPT Timeout|0.00|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |Not serializable|9.87|N/A|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.99|✅|
| `nondet` |SMPT Timeout|0.00|N/A|
| `nondet2` |Not serializable|8.52|✅|
| `nondet_impl` |Not serializable|5.93|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|3.42|✅|
| `self_loop2` |Serializable|6.62|✅|
| `simple_nonser` |Not serializable|4.23|✅|
| `simple_nonser2` |Not serializable|3.64|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.96|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|4.34|❌|
| `simple_nonser3` |Not serializable|3.82|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |SMPT Timeout|0.00|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|7.65|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 8 (valid proofs: 6, invalid: 2)
- Not serializable: 11 (valid traces: 8, invalid: 3)
- Timeouts: 30, Errors: 1, Total: 50
