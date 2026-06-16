| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `type bench\input\small.md \| .\target\release\marked-rs.exe` | 30.6 ± 3.5 | 26.4 | 42.5 | 1.00 |
| `type bench\input\small.md \| node bench.mjs` | 123.3 ± 12.5 | 105.6 | 161.3 | 4.03 ± 0.62 |
| `type bench\input\medium.md \| .\target\release\marked-rs.exe` | 56.7 ± 7.8 | 45.2 | 87.0 | 1.85 ± 0.33 |
| `type bench\input\medium.md \| node bench.mjs` | 168.5 ± 20.2 | 147.2 | 258.0 | 5.51 ± 0.92 |
| `type bench\input\large.md \| .\target\release\marked-rs.exe` | 181.3 ± 16.4 | 157.3 | 244.1 | 5.93 ± 0.87 |
| `type bench\input\large.md \| node bench.mjs` | 434.1 ± 52.1 | 374.8 | 574.1 | 14.20 ± 2.36 |
