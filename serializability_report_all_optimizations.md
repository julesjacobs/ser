# Serializability Analysis Report - _All_Optimizations
Generated: 2025-07-02 00:13:11
Extras: []

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|7.31|✅|
| `login_flow` |SMPT Timeout|0.00|N/A|
| `shopping_cart` |SMPT Timeout|0.00|N/A|
| `state_machine` |SMPT Timeout|0.00|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|3.25|❌|
| `flag_non_ser` |Not serializable|3.62|✅|
| `flag_non_ser_turned_ser` |Serializable|3.02|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_2` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky2` |Not serializable|5.19|✅|
| `fred_arith_tricky3` |Not serializable|9.84|✅|
| `funny` |Not serializable|2.90|✅|
| `if_while_with_req` |Serializable|2.08|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |SMPT Timeout|0.00|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |Not serializable|3.44|✅|
| `multiple_requests_updated` |SMPT Timeout|0.00|N/A|
| `nested_while` |Serializable|2.14|✅|
| `nondet` |Serializable|4.06|✅|
| `nondet2` |Not serializable|7.10|✅|
| `nondet_impl` |Not serializable|3.31|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.54|✅|
| `self_loop2` |Serializable|5.88|✅|
| `simple_nonser` |Not serializable|4.40|✅|
| `simple_nonser2` |Not serializable|4.14|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|3.66|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|4.42|❌|
| `simple_nonser3` |Not serializable|4.00|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |SMPT Timeout|0.00|N/A|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|9.75|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|5.68|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 10 (valid proofs: 8, invalid: 2)
- Not serializable: 12 (valid traces: 11, invalid: 1)
- Timeouts: 28, Errors: 0, Total: 50
