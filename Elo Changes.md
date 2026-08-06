#  Elo history.

Baseline: PVT implementation. (Commit 56d2375d88cb1a09b0764d7f16441373cdce1461)

TT move ordering implementation: dcfb7cc9d8e18af5e677d7dcea80ef084c18716e
After TT move ordering implementation: Approximately 160 Elo gained.


```Results of tt-ordering vs psts (10+0.2, NULL, NULL, UHO_4060_v4.epd):
Elo: 162.63 +/- 13.49, nElo: 207.38 +/- 14.92
LOS: 100.00 %, DrawRatio: 26.10 %, PairsRatio: 6.62
Games: 2084, Wins: 1241, Losses: 331, Draws: 512, Points: 1497.0 (71.83 %)
Ptnml(0-2): [19, 82, 272, 308, 361], WL/DD Ratio: 3.46
```

TT cutoffs implementation: 94e29c70392786ab82b539168007dd7ec7aa9cc9
After TT cutoffs implementation: Approximately 30 Elo gained. 

```
Results of tt-cutoffs vs tt-ordering (10+0.2, NULL, NULL, UHO_4060_v4.epd):
Elo: 31.71 +/- 14.63, nElo: 53.96 +/- 24.73
LOS: 100.00 %, DrawRatio: 61.21 %, PairsRatio: 2.13
Games: 758, Wins: 305, Losses: 236, Draws: 217, Points: 413.5 (54.55 %)
Ptnml(0-2): [13, 34, 232, 71, 29], WL/DD Ratio: 3.14
LLR: 2.95 (100.3%) (-2.94, 2.94) [0.00, 10.00]
```