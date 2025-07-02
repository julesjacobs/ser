# Serializability Analysis Report - _All_Optimizations
Generated: 2025-07-02 01:24:12
Extras: []

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|6.58|✅|
| `login_flow` |Serializable|23.82|✅|
| `shopping_cart` |SMPT Timeout|0.00|N/A|
| `state_machine` |Serializable|25.85|✅|
| `BGP_routing` |Not serializable|49.51|✅|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |Not serializable|42.69|N/A|
| `complex_while_with_yields` |Not serializable|41.66|N/A|
| `ex` |Serializable|4.00|❌|
| `flag_non_ser` |Not serializable|4.09|✅|
| `flag_non_ser_turned_ser` |Serializable|3.46|✅|
| `fred2_arith` |Not serializable|39.42|N/A|
| `fred_arith` |SMPT Timeout|0.00|N/A|
| `fred_arith_simplified_until_1` |Serializable|15.81|✅|
| `fred_arith_simplified_until_2` |SMPT Timeout|0.00|N/A|
| `fred_arith_tricky` |Not serializable|33.82|✅|
| `fred_arith_tricky2` |Not serializable|4.40|✅|
| `fred_arith_tricky3` |Not serializable|6.69|✅|
| `funny` |Not serializable|2.84|✅|
| `if_while_with_req` |Serializable|2.43|✅|
| `incrdecr` |SMPT Timeout|0.00|N/A|
| `less_simple_ser` |SMPT Timeout|0.00|N/A|
| `modulo` |Serializable|35.83|✅|
| `modulo_nonser` |Not serializable|3.04|✅|
| `multiple_requests_updated` |Serializable|6.37|✅|
| `nested_while` |Serializable|2.34|✅|
| `nondet` |Serializable|2.86|✅|
| `nondet2` |Not serializable|3.46|✅|
| `nondet_impl` |Not serializable|2.66|✅|
| `nondet_impl2` |Serializable|48.03|✅|
| `self_loop` |Serializable|2.19|✅|
| `self_loop2` |Serializable|3.04|✅|
| `simple_nonser` |Not serializable|2.57|✅|
| `simple_nonser2` |Not serializable|2.44|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.01|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|3.08|❌|
| `simple_nonser3` |Not serializable|3.02|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|17.58|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|9.80|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|33.60|N/A|
| `stop2` |Serializable|30.05|✅|
| `stop3` |Not serializable|4.08|✅|
| `stop3a` |Not serializable|34.17|N/A|
| `stop4` |Serializable|48.47|✅|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|40.90|✅|
| `tricky3` |Not serializable|52.50|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 18 (valid proofs: 16, invalid: 2)
- Not serializable: 22 (valid traces: 16, invalid: 6)
- Timeouts: 10, Errors: 0, Total: 50
