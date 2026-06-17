hyperfine --warmup 10 --runs 50 --export-markdown bench/comparison.md `
  'type bench\input\small.md | .\target\release\marked-rs.exe' `
  'type bench\input\small.md | node bench.mjs' `
  'type bench\input\medium.md | .\target\release\marked-rs.exe' `
  'type bench\input\medium.md | node bench.mjs' `
  'type bench\input\large.md | .\target\release\marked-rs.exe' `
  'type bench\input\large.md | node bench.mjs'
