# Serializability Analysis Report - _Only_Smart_Kleene
Generated: 2025-07-02 01:32:15
Extras: ['--without-bidirectional', '--without-remove-redundant', '--without-generate-less']

|Example|Result|CPU(s)|Valid?|
|--|--|--|--|
| `data_flow` |Serializable|9.03|✅|
| `login_flow` |Error|3.01|N/A|
| `shopping_cart` |Error|3.64|N/A|
| `state_machine` |Error|3.27|N/A|
| `BGP_routing` |SMPT Timeout|0.00|N/A|
| `bank` |SMPT Timeout|0.00|N/A|
| `bank_without_yields` |SMPT Timeout|0.00|N/A|
| `complex_while_with_yields` |Not serializable|44.10|N/A|
| `ex` |Serializable|3.84|❌|
| `flag_non_ser` |Not serializable|33.23|N/A|
| `flag_non_ser_turned_ser` |Serializable|2.99|✅|
| `fred2_arith` |Not serializable|42.25|N/A|
| `fred_arith` |Not serializable|36.52|N/A|
| `fred_arith_simplified_until_1` |Serializable|32.95|✅|
| `fred_arith_simplified_until_2` |Not serializable|33.91|N/A|
| `fred_arith_tricky` |Not serializable|35.60|N/A|
| `fred_arith_tricky2` |Not serializable|5.39|✅|
| `fred_arith_tricky3` |Not serializable|9.21|✅|
| `funny` |Not serializable|2.90|✅|
| `if_while_with_req` |Serializable|2.36|✅|
| `incrdecr` |Not serializable|34.06|N/A|
| `less_simple_ser` |Error|3.11|N/A|
| `modulo` |SMPT Timeout|0.00|N/A|
| `modulo_nonser` |SMPT Timeout|0.00|N/A|
| `multiple_requests_updated` |Serializable|11.84|✅|
| `nested_while` |Serializable|2.48|✅|
| `nondet` |Serializable|5.66|❌|
| `nondet2` |Not serializable|5.02|✅|
| `nondet_impl` |Not serializable|3.14|✅|
| `nondet_impl2` |SMPT Timeout|0.00|N/A|
| `self_loop` |Serializable|2.51|✅|
| `self_loop2` |Serializable|4.37|✅|
| `simple_nonser` |Not serializable|2.93|✅|
| `simple_nonser2` |Not serializable|2.51|✅|
| `simple_nonser2_minus_yields_is_ser` |Serializable|2.11|✅|
| `simple_nonser2_turned_ser_with_locks` |Serializable|2.78|❌|
| `simple_nonser3` |Not serializable|2.71|✅|
| `snapshot_isolation_network_monitoring` |Not serializable|28.37|✅|
| `snapshot_isolation_network_monitoring_without_yields` |SMPT Timeout|0.00|N/A|
| `stateful_firewall` |Not serializable|11.05|✅|
| `stateful_firewall_without_yields` |SMPT Timeout|0.00|N/A|
| `stop` |Not serializable|34.46|N/A|
| `stop2` |Serializable|59.52|✅|
| `stop3` |Not serializable|5.84|✅|
| `stop3a` |Not serializable|36.36|N/A|
| `stop4` |SMPT Timeout|0.00|N/A|
| `stop4a` |SMPT Timeout|0.00|N/A|
| `tricky2` |Not serializable|46.70|N/A|
| `tricky3` |Not serializable|58.89|N/A|
| `tricky3_ser` |SMPT Timeout|0.00|N/A|

## Summary
- Serializable: 13 (valid proofs: 10, invalid: 3)
- Not serializable: 22 (valid traces: 11, invalid: 11)
- Timeouts: 11, Errors: 4, Total: 50
