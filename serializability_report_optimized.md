# Serializability Analysis Report - _Optimized
Generated: 2025-07-02 00:05:52
Extras: []

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|4.29|✅|
| `login_flow` |SMPT Timeout|0.00|N/A|
| `shopping_cart` |SMPT Timeout|0.00|N/A|
| `state_machine` |SMPT Timeout|0.00|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |SMPT Timeout|0.00|N/A|
| `ex` |Serializable|2.90|❌|
| `flag_non_ser` |Not serializable|3.03|✅|
| `flag_non_ser_turned_ser` |Serializable|2.42|✅|
| `fred2_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |Serializable|9.38|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky` |Not serializable|8.42|N/A|
| `fred_arith_tricky2` |Not serializable|3.27|✅|
| `fred_arith_tricky3` |Not serializable|4.64|✅|
| `funny` |Not serializable|2.55|✅|
| `if_while_with_req` |Serializable|1.90|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |SMPT Timeout|0.00|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |Not serializable|2.83|✅|
| `multiple_requests_updated` |Serializable|8.33|✅|
| `nested_while` |Serializable|2.15|✅|
| `nondet` |Serializable|3.64|✅|
| `nondet2` |Not serializable|4.98|✅|
| `nondet_impl` |Not serializable|3.25|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.48|✅|
| `self_loop2` |Serializable|4.42|✅|
| `simple_nonser` |Not serializable|3.58|✅|
| `simple_nonser2` |Not serializable|3.16|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.93|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.48|❌|
| `simple_nonser3` |Not serializable|3.32|✅|
| `snapshot_isolation_network_monitoring` |SMPT Timeout|0.00|N/A|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|9.58|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|8.82|N/A|
| `stop2` |SMPT Timeout|0.00|N/A|
| `stop3` |Not serializable|4.57|✅|
| `stop3a` |SMPT Timeout|0.00|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |SMPT Timeout|0.00|N/A|
| `tricky3` |SMPT Timeout|0.00|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 12 (valid proofs: 10, invalid: 2)
- Not serializable: 14 (valid traces: 12, invalid: 2)
- Timeouts: 24, Errors: 0, Total: 50
