# From Binary to Ternary: Data Pipelines

## The Trap

Binary data pipelines treat every item as pass or fail. A filter keeps it or drops it. A transform maps it or errors. An aggregator includes it or skips it. There is no middle ground — either the data flows through every stage cleanly or it's rejected.

Real data processing doesn't work this way. Records arrive with missing fields. Sensors send corrupted readings that are still informative. A log line might be malformed but contain useful timestamps. Binary pipelines discard this data. Ternary pipelines can keep it, flag it, and let downstream stages decide.

## Map to Three States

| Domain | −1 | 0 | +1 |
|--------|----|---|-----|
| Item status | rejected | uncertain / flag | accepted |
| Filter result | exclude | review | include |
| Transform outcome | error | warning | success |
| Aggregate treatment | ignore | partial weight | full weight |

## From Binary to Ternary

**Before: binary pipeline stage**

```rust
trait Stage<I, O> {
    fn process(&self, input: I) -> Option<O>;
    // None = reject, Some = pass
    // What about "pass, but with warnings"?
}
```

Every stage can only accept or reject. If a stage needs to say "this worked but something's odd," it must either reject the item (losing data) or pass it silently (hiding the issue).

**After: ternary pipeline result**

```rust
enum PipelineResult<T> {
    Positive(T),       // +1: accepted, clean
    Neutral(T, Vec<Warning>),  // 0: accepted with caveats
    Negative(Error),   // -1: rejected
}

trait Stage<I, O> {
    fn process(&self, input: I) -> PipelineResult<O>;
}
```

The `Neutral` state is the killer feature. A parser that encounters a malformed JSON string but can still extract 80% of the fields returns `Neutral(parsed, vec![Warning::MalformedField("timestamp")])`. Downstream stages see the warning and adjust: the aggregation stage gives this record partial weight instead of ignoring it entirely.

**0 is not nothing:** In a binary pipeline, an uncertain record is either forced through (risking downstream corruption) or dropped (losing partial information). In a ternary pipeline, `Neutral` is an active signal — it says "I got something, but treat it carefully." Downstream stages can branch on the result type:

```rust
match pipeline.process(&item) {
    PipelineResult::Positive(data) => aggregate_weighted(data, 1.0),
    PipelineResult::Neutral(data, _) => aggregate_weighted(data, 0.5),
    PipelineResult::Negative(_) => {},  // skip
}
```

**The ternary conservation law** applies to pipeline throughput: in a closed processing pipeline, the sum of accepted, neutral, and rejected items equals the input count. Every item goes somewhere — nothing is silently lost.

**Before: pipeline builder with binary stages**

```rust
PipelineBuilder::new()
    .add(FilterStage::positive_only())
    .add(TransformStage::new(|x| x * 2))
    .build()
```

**After: pipeline with ternary-aware composition**

```rust
PipelineBuilder::new()
    .add(FilterStage::new(Ternary::Positive))  // only positive items
    .add(TransformStage::new(|x| x * 2))
    .add(SortStage::ternary())                  // sort by ternary value
    .build()
```

Stages can filter on ternary state, transform neutral items differently from positive ones, or aggregate with ternary-weighted contributions.

## Why It Matters

Ternary pipelines don't throw away uncertain data. The `Neutral` state preserves partial information, lets downstream stages make informed decisions, and prevents the binary all-or-nothing data loss that plagues production data systems. Your data is rarely perfectly clean — ternary pipelines are honest about that.
