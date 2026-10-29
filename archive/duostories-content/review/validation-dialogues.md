# Validation report

- sentences: 4011
- doubtful: 64
- parse: 0
- morph: 0
- unknown: 0
- name: 0
- type: 0
- type_checker: 469
- doubtful_parse: 0
- doubtful_morph: 0
- doubtful_unknown: 8
- doubtful_name: 0
- doubtful_type: 10
- doubtful_type_checker: 8
- failing_sentences: 0
- unique_unknown_words: 0
- unique_invalid_names: 0

## Failing (non-doubtful)


## Doubtful with issues

- `item 82` phrase: `mi pilno la lensisku le nu mi tadni la .lojban.`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `item 83` phrase: `xu le nu do pilno la lensisku cu xamgu do`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `item 84` phrase: `mi tatpi le nu mi pilno la lensisku`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `item 389` phrase: `.ai mi klama le tcadu fu tu'a le taksi`
  - unknown: taksi: experimental gismu
- `item 401` phrase: `na go'i .i mi ba klama fu tu'a le taksi`
  - unknown: taksi: experimental gismu
- `item 473` line 0: `mi pilno la lensisku le nu mi tadni la .lojban.`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `item 473` line 1: `.i xu le nu do pilno la lensisku cu xamgu do`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `item 473` line 3: `.i ku'i mi tatpi le nu mi pilno la lensisku`
  - unknown: lensisku: lujvo not in lujvo.tsv
- `batch2-03` line 9: `.i la .lilianas. cu lebna lo bolci gi'e renro ri fi lo kelci vorme`
  - type_checker: x2 of 'lebna' expects Set<T>, but got lo bolci (Entity)
- `batch2-06` line 11: `.i mu'i lo nu mi ba cpedu lo nu gunka`
  - type_checker: x2 of 'cpedu' expects Set<V>, but got lo nu (Abstraction: (Event))
- `batch2-07` line 3: `.i do nanca li xo`
  - type: x1 of 'nanca' expects TimeInterval, but got do
- `batch2-07` line 4: `.i mi nanca li cimu`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `batch2-07` line 8: `.i ku'i mi se cmene zoi gy. Emilia Rossi gy. .i mi nanca li cimu`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `batch2-07` q1 choice: `.i la .emilias. cu nanca li xo`
  - type: x1 of 'nanca' expects TimeInterval, but got la emilias
- `batch2-19` line 8: `.i mi dukse lo ka terpa`
  - type_checker: x2 of 'dukse' expects T, but got lo ka (Abstraction: (Set<Entity>))
- `batch2-39` line 4: `.i ja'o mi na ba nanca li cino`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `lesson-23` phrase 3: `mi'o klama fu lo karce`
  - type_checker: x5 of 'klama' expects Set<T>, but got lo karce (Entity)
- `item 63` line 4: `.i ja'o mi na ba nanca li cino`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `item 21` line 11: `.i mu'i lo nu mi ba cpedu lo nu gunka`
  - type_checker: x2 of 'cpedu' expects Set<V>, but got lo nu (Abstraction: (Event))
- `item 26` line 3: `.i do nanca li xo`
  - type: x1 of 'nanca' expects TimeInterval, but got do
- `item 26` line 4: `.i mi nanca li cimu`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `item 26` line 8: `.i ku'i mi se cmene zoi gy. Emilia Rossi gy. .i mi nanca li cimu`
  - type: x1 of 'nanca' expects TimeInterval, but got mi
- `item 26` q1 choice: `.i la .emilias. cu nanca li xo`
  - type: x1 of 'nanca' expects TimeInterval, but got la emilias
- `item 14` line 9: `.i la .lilianas. cu lebna lo bolci gi'e renro ri fi lo kelci vorme`
  - type_checker: x2 of 'lebna' expects Set<T>, but got lo bolci (Entity)
- `item 42` line 8: `.i mi dukse lo ka terpa`
  - type_checker: x2 of 'dukse' expects T, but got lo ka (Abstraction: (Set<Entity>))
- `item 205` phrase: `mi'o klama fu lo karce`
  - type_checker: x5 of 'klama' expects Set<T>, but got lo karce (Entity)
