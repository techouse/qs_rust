# Performance Comparison

Status: `partial`

Captured: `2026-03-23T09:03:54.805048+00:00`

This summary is informative only. Cross-language numbers are machine- and command-specific.

## Encode

| Language | Runtime | Depth | ms/op | Length | Alloc/op |
| --- | --- | ---: | ---: | ---: | ---: |
| csharp | csharp | 2000 | 0.109 | n/a | n/a |
| dart | dart | 2000 | 0.209 | 6006 | n/a |
| kotlin | kotlin | 2000 | 0.223 | 6006 | 283115 |
| python | python | 2000 | 1.293 | n/a | n/a |
| rust | rust | 2000 | 0.100 | 6006 | n/a |
| swift | objc | 2000 | 1.173 | 6006 | n/a |
| swift | swift | 2000 | 0.237 | 6006 | n/a |
| csharp | csharp | 5000 | 0.295 | n/a | n/a |
| dart | dart | 5000 | 0.512 | 15006 | n/a |
| kotlin | kotlin | 5000 | 0.279 | 15006 | 681574 |
| python | python | 5000 | 3.205 | n/a | n/a |
| rust | rust | 5000 | 0.159 | 15006 | n/a |
| swift | objc | 5000 | 2.437 | 15006 | n/a |
| swift | swift | 5000 | 0.676 | 15006 | n/a |
| csharp | csharp | 12000 | 0.949 | n/a | n/a |
| dart | dart | 12000 | 1.325 | 36006 | n/a |
| kotlin | kotlin | 12000 | 0.865 | 36006 | 1845493 |
| python | python | 12000 | 7.875 | n/a | n/a |
| rust | rust | 12000 | 0.305 | 36006 | n/a |
| swift | objc | 12000 | 5.729 | 36006 | n/a |
| swift | swift | 12000 | 1.561 | 36006 | n/a |

## Decode

| Language | Runtime | Case | Count | Comma | UTF8 | Len | ms/op | Keys | Alloc/op |
| --- | --- | --- | ---: | --- | --- | ---: | ---: | ---: | ---: |
| csharp | csharp | C1 | 100 | false | false | 8 | 0.009 | 100 | n/a |
| dart | dart | C1 | 100 | false | false | 8 | 0.030 | 100 | n/a |
| kotlin | kotlin | C1 | 100 | false | false | 8 | 0.029 | 100 | 30515 |
| python | python | C1 | 100 | false | false | 8 | 0.092 | 100 | n/a |
| rust | rust | C1 | 100 | false | false | 8 | 0.013 | 100 | n/a |
| swift | objc | C1 | 100 | false | false | 8 | 0.077 | 100 | n/a |
| swift | swift | C1 | 100 | false | false | 8 | 0.050 | 100 | n/a |
| csharp | csharp | C2 | 1000 | false | false | 40 | 0.118 | 1000 | n/a |
| dart | dart | C2 | 1000 | false | false | 40 | 0.539 | 1000 | n/a |
| kotlin | kotlin | C2 | 1000 | false | false | 40 | 0.302 | 1000 | 392089 |
| python | python | C2 | 1000 | false | false | 40 | 1.055 | 1000 | n/a |
| rust | rust | C2 | 1000 | false | false | 40 | 0.152 | 1000 | n/a |
| swift | objc | C2 | 1000 | false | false | 40 | 0.835 | 1000 | n/a |
| swift | swift | C2 | 1000 | false | false | 40 | 0.784 | 1000 | n/a |
| csharp | csharp | C3 | 1000 | true | true | 40 | 0.143 | 1000 | n/a |
| dart | dart | C3 | 1000 | true | true | 40 | 0.540 | 1000 | n/a |
| python | python | C3 | 1000 | true | true | 40 | 1.158 | 1000 | n/a |
| rust | rust | C3 | 1000 | true | true | 40 | 0.141 | 1000 | n/a |
| swift | objc | C3 | 1000 | true | true | 40 | 0.875 | 1000 | n/a |
| swift | swift | C3 | 1000 | true | true | 40 | 0.817 | 1000 | n/a |

## Commands

- `rust/encode`: `cargo run --release --bin qs_perf -- --scenario encode --format json` in `~/Work/qs_rust` (`rc=0`)
- `rust/decode`: `cargo run --release --bin qs_perf -- --scenario decode --format json` in `~/Work/qs_rust` (`rc=0`)
- `python/encode`: `python3 scripts/bench_encode_depth.py --runs 7 --warmups 5` in `~/Work/qs.py` (`rc=0`)
- `python/decode`: `python3 scripts/bench_decode_snapshot.py --samples 7 --warmups 5` in `~/Work/qs.py` (`rc=0`)
- `dart/encode`: `dart run tool/perf_snapshot.dart` in `~/Work/qs.dart` (`rc=0`)
- `dart/decode`: `dart run tool/decode_perf_snapshot.dart` in `~/Work/qs.dart` (`rc=0`)
- `kotlin/all`: `./gradlew :comparison:run --args perf` in `~/Work/qs-kotlin` (`rc=0`)
- `swift/encode`: `swift run -c release QsSwiftBench perf` in `~/Work/QsSwift/Bench` (`rc=0`)
- `swift/decode`: `swift run -c release QsSwiftBench perf-decode` in `~/Work/QsSwift/Bench` (`rc=0`)
- `csharp/encode`: `dotnet run -c Release --project benchmarks/QsNet.Benchmarks -- --filter *Encode_DeepNesting*` in `~/Work/QsNet` (`rc=0`)
- `csharp/decode`: `dotnet run -c Release --project benchmarks/QsNet.Benchmarks -- --filter *Decode_Public*` in `~/Work/QsNet` (`rc=0`)

## Failures

- `kotlin/all`: coverage failed: missing canonical decode cases: C3
