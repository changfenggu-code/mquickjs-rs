# Latest Benchmark Results

- This file keeps the latest result for each benchmark view.
- Each section is overwritten independently when its corresponding command runs.

## Rust Criterion

<!-- BEGIN criterion -->
- Generated at: `2026-04-01 22:41:46 +0800`
- Metric: `Criterion Rust-only runtime benchmark`
- Selection: `full js_benchmarks bench target`

| Benchmark | Criterion Time |
|-----------|----------------|
| fib_iter 1k | [2.5278 ms 2.7253 ms 2.9275 ms] |
| loop 10k | [460.27 µs 507.58 µs 558.17 µs] |
| array push 10k | [1.0849 ms 1.2025 ms 1.3266 ms] |
| object create 1k | [782.40 µs 873.56 µs 970.70 µs] |
| closure 1k | [551.61 µs 607.66 µs 668.69 µs] |
| string concat 1k | [100.57 µs 112.04 µs 123.76 µs] |
| json parse only 1k | [1.6807 ms 1.8528 ms 2.0211 ms] |
| math max 3arg 10k | [2.1589 ms 2.3345 ms 2.5149 ms] |
| sieve 10k | [6.0175 ms 6.5101 ms 7.0148 ms] |
| recursion 100x100 | [910.31 µs 1.0073 ms 1.1143 ms] |
| switch 1k | [125.63 µs 140.45 µs 155.67 µs] |
| do...while 10k | [259.07 µs 286.13 µs 315.31 µs] |
| method_chain 5k | [1.4458 ms 1.5949 ms 1.7529 ms] |
| for_of_array 20k | [2.9695 ms 3.2234 ms 3.4857 ms] |
| deep_property 200k | [18.280 ms 19.665 ms 21.088 ms] |
| try_catch 5k | [406.20 µs 496.93 µs 624.78 µs] |
| for_in_object 20x2000 | [6.8437 ms 7.4255 ms 8.0275 ms] |
<!-- END criterion -->

## Rust-vs-C Execution-Only

<!-- BEGIN execution_only -->
- Generated at: `2026-04-02 00:11:09 +0800`
- Platform: `MINGW64_NT-10.0-26200`
- Python: `Python 3.12.6`
- Mode: `non-canonical`
- Metric: `compile-once / execute-many average (50 in-process iterations)`
- Selection: `all workloads`
- Rust runner: `/d/projects/mquickjs-rs/target/release/bench_exec_helper.exe`
- C runner: `/d/projects/mquickjs-rs/target/bench-tools/mqjs-build/bench_exec_helper.exe`

| Benchmark | Rust (s) | C (s) | Ratio | Notes |
|-----------|----------|-------|-------|-------|
| array | 0.002030 | 0.000938 | 2.16x | C faster |
| array_push_10k | 0.001504 | 0.000594 | 2.53x | C faster |
| closure | 0.000694 | 0.000173 | 4.01x | C faster |
| closure_1k | 0.000522 | 0.000109 | 4.79x | C faster |
| deep_property | 0.015184 | 0.009881 | 1.54x | C faster |
| dense_array_bool_condition_only_hot | 0.255895 | 0.068193 | 3.75x | C faster |
| dense_array_bool_condition_only_hot_arg0 | 0.230846 | 0.067432 | 3.42x | C faster |
| dense_array_bool_condition_only_hot_local1 | 0.224167 | 0.066828 | 3.35x | C faster |
| dense_array_bool_read_branch_10k | 0.002293 | 0.001110 | 2.07x | C faster |
| dense_array_bool_read_hot | 0.246414 | 0.085949 | 2.87x | C faster |
| dense_array_false_write_only_10k | 0.003996 | 0.001200 | 3.33x | C faster |
| dense_array_false_write_then_read_hot | 0.227967 | 0.066014 | 3.45x | C faster |
| dense_array_loop_only_hot | 0.050299 | 0.044223 | 1.14x | C faster |
| dense_array_read_only_hot | 0.223652 | 0.063200 | 3.54x | C faster |
| dense_array_read_only_hot_arg0 | 0.222774 | 0.064770 | 3.44x | C faster |
| dense_array_read_only_hot_local1 | 0.222004 | 0.062916 | 3.53x | C faster |
| do_while_10k | 0.000203 | 0.000213 | 0.95x | ~Equal |
| fib | 0.224825 | 0.102529 | 2.19x | C faster |
| fib_iter | 0.001804 | 0.001553 | 1.16x | C faster |
| for_in_object | 0.006024 | 0.001282 | 4.70x | C faster |
| for_of_array | 0.002689 | 0.001619 | 1.66x | C faster |
| gc_transient_arrays_2500 | 0.001017 | 0.000224 | 4.54x | C faster |
| json_parse_only | 0.001657 | 0.001335 | 1.24x | C faster |
| json_parse_property_read | 0.001603 | 0.001576 | 1.02x | ~Equal |
| loop | 0.056393 | 0.040249 | 1.40x | C faster |
| loop_10k | 0.000409 | 0.000324 | 1.26x | C faster |
| math_max_3arg_10k | 0.001473 | 0.000747 | 1.97x | C faster |
| method_chain | 0.001272 | 0.000771 | 1.65x | C faster |
| object | 0.008674 | 0.003828 | 2.27x | C faster |
| object_create_1k | 0.000669 | 0.000228 | 2.93x | C faster |
| recursion_100x100 | 0.000857 | 0.000397 | 2.16x | C faster |
| runtime_string_pressure | 0.001741 | 0.001991 | 0.87x | Rust faster |
| sieve | 0.046355 | 0.015710 | 2.95x | C faster |
| sieve_10k | 0.003864 | 0.001403 | 2.75x | C faster |
| string | 0.001502 | 0.000224 | 6.71x | C faster |
| string_concat_1k | 0.000107 | 0.000147 | 0.73x | Rust faster |
| string_concat_ephemeral | 0.000276 | 0.000092 | 3.00x | C faster |
| string_concat_local_update_only | 0.000087 | 0.000034 | 2.56x | C faster |
| switch_1k | 0.000124 | 0.000093 | 1.33x | C faster |
| switch_case | 0.001767 | 0.002786 | 0.63x | Rust faster |
| try_catch | 0.000372 | 0.000285 | 1.31x | C faster |

## Notes

- This section is updated by `benches/compare.sh`.
<!-- END execution_only -->

## Rust-vs-C End-to-End

<!-- BEGIN end_to_end -->
- Generated at: `2026-04-01 23:09:18 +0800`
- Platform: `MINGW64_NT-10.0-26200`
- Python: `Python 3.12.6`
- Mode: `non-canonical`
- Metric: `end-to-end process execution average (15 process runs)`
- Selection: `all workloads`
- Rust runner: `/d/projects/mquickjs-rs/target/release/mqjs.exe`
- C runner: `/d/projects/mquickjs-rs/target/bench-tools/mqjs-build/mqjs/mqjs.exe`

| Benchmark | Rust (s) | C (s) | Ratio | Notes |
|-----------|----------|-------|-------|-------|
| array | 0.1510 | 0.1522 | 0.99x | ~Equal |
| array_push_10k | 0.1414 | 0.1423 | 0.99x | ~Equal |
| closure | 0.1509 | 0.1443 | 1.05x | ~Equal |
| closure_1k | 0.1443 | 0.1451 | 0.99x | ~Equal |
| deep_property | 0.1600 | 0.1466 | 1.09x | ~Equal |
| dense_array_bool_condition_only_hot | 0.3997 | 0.1418 | 2.82x | C faster |
| dense_array_bool_condition_only_hot_arg0 | 0.4062 | 0.1420 | 2.86x | C faster |
| dense_array_bool_condition_only_hot_local1 | 0.4026 | 0.1370 | 2.94x | C faster |
| dense_array_bool_read_branch_10k | 0.1505 | 0.1420 | 1.06x | ~Equal |
| dense_array_bool_read_hot | 0.4170 | 0.1371 | 3.04x | C faster |
| dense_array_false_write_only_10k | 0.1427 | 0.1395 | 1.02x | ~Equal |
| dense_array_false_write_then_read_hot | 0.3967 | 0.1445 | 2.75x | C faster |
| dense_array_loop_only_hot | 0.1931 | 0.1401 | 1.38x | C faster |
| dense_array_read_only_hot | 0.3961 | 0.1488 | 2.66x | C faster |
| dense_array_read_only_hot_arg0 | 0.3953 | 0.1346 | 2.94x | C faster |
| dense_array_read_only_hot_local1 | 0.3944 | 0.1476 | 2.67x | C faster |
| do_while_10k | 0.1525 | 0.1353 | 1.13x | C faster |
| fib | 0.3768 | 0.2617 | 1.44x | C faster |
| fib_iter | 0.1481 | 0.1504 | 0.98x | ~Equal |
| for_in_object | 0.1558 | 0.1433 | 1.09x | ~Equal |
| for_of_array | 0.1501 | 0.1403 | 1.07x | ~Equal |
| gc_transient_arrays_2500 | 0.1480 | 0.1458 | 1.02x | ~Equal |
| json_parse_only | 0.1438 | 0.1521 | 0.95x | ~Equal |
| json_parse_property_read | 0.1536 | 0.1474 | 1.04x | ~Equal |
| loop | 0.2042 | 0.1829 | 1.12x | C faster |
| loop_10k | 0.1461 | 0.1411 | 1.04x | ~Equal |
| math_max_3arg_10k | 0.1478 | 0.1417 | 1.04x | ~Equal |
| method_chain | 0.1558 | 0.1394 | 1.12x | C faster |
| object | 0.1553 | 0.1490 | 1.04x | ~Equal |
| object_create_1k | 0.1506 | 0.1383 | 1.09x | ~Equal |
| recursion_100x100 | 0.1490 | 0.1434 | 1.04x | ~Equal |
| runtime_string_pressure | 0.1454 | 0.1487 | 0.98x | ~Equal |
| sieve | 0.1939 | 0.1612 | 1.20x | C faster |
| sieve_10k | 0.1479 | 0.1419 | 1.04x | ~Equal |
| string | 0.1480 | 0.1429 | 1.04x | ~Equal |
| string_concat_1k | 0.1515 | 0.1445 | 1.05x | ~Equal |
| string_concat_ephemeral | 0.1453 | 0.1432 | 1.01x | ~Equal |
| string_concat_local_update_only | 0.1533 | 0.1467 | 1.04x | ~Equal |
| switch_1k | 0.1528 | 0.1467 | 1.04x | ~Equal |
| switch_case | 0.1527 | 0.1475 | 1.04x | ~Equal |
| try_catch | 0.1470 | 0.1435 | 1.02x | ~Equal |

## Notes

- This section is updated by `benches/compare.sh`.
<!-- END end_to_end -->
