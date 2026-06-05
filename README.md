# ternary-pipeline: Composable data processing pipelines for ternary items

Chain filter, transform, aggregate, sort, and limit stages into a pipeline that processes items carrying ternary values (−1, 0, +1). Stages execute sequentially; errors halt the pipeline and report which stage failed.

## Why This Exists

Processing ternary data often involves the same patterns: filter out neutrals, negate values, sort by magnitude, take the top N, then aggregate. Writing these as separate loops is repetitive and error-prone. This crate gives you a typed pipeline abstraction where each stage is a separate, testable unit, and the composition is explicit and inspectable.

## Core Concepts

- **TernaryItem** — A data record with an `id` (usize), a `value` (i8: −1, 0, or +1), and a `label` (String). The unit of data flowing through the pipeline.
- **Stage** — A trait for pipeline processing steps. Each stage takes a `Vec<TernaryItem>` and returns `Result<Vec<TernaryItem>, String>`. Stages can filter, transform, or aggregate.
- **Pipeline** — An ordered list of `Box<dyn Stage>` instances. Calling `run()` executes them sequentially. If any stage returns `Err`, the pipeline stops and reports the error.
- **PipelineResult** — The output of a pipeline run: the final items, a list of stages that executed, and success/error status.
- **PipelineBuilder** — A fluent API for constructing pipelines. Chain `.filter()`, `.transform()`, `.sort()`, `.limit()`, `.aggregate()` calls, then `.build()`.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-pipeline = "0.1"
```

```rust
use ternary_pipeline::*;

let input = vec![
    TernaryItem::new(1, 1, "buy"),
    TernaryItem::new(2, -1, "sell"),
    TernaryItem::new(3, 0, "hold"),
    TernaryItem::new(4, 1, "buy"),
];

let pipeline = PipelineBuilder::new()
    .filter("non_neutral", |item| !item.is_neutral())
    .transform("negate", |item| TernaryItem::new(item.id, -item.value, &item.label))
    .sort()
    .limit(2)
    .build();

let result = pipeline.run(input);
assert!(result.success);
assert_eq!(result.stages_run, vec!["non_neutral", "transform_negate", "sort", "limit"]);
assert_eq!(result.items.len(), 2);
println!("{}", result);
```

## API Overview

| Type | What it is |
|---|---|
| `TernaryItem` | Data record: id, value (−1/0/+1), label |
| `Stage` | Trait: `name()` + `process(items) → Result` |
| `FilterStage` | Keeps items matching a predicate |
| `TransformStage` | Maps each item to a new item |
| `AggregateStage` | Reduces all items to one (sum or count) |
| `SortStage` | Sorts items by value ascending |
| `LimitStage` | Takes at most N items |
| `Pipeline` | Ordered stage list; `run()` executes sequentially |
| `PipelineBuilder` | Fluent builder for constructing pipelines |
| `PipelineResult` | Output: items, stages run, success/error |

## How It Works

**Stage execution.** `Pipeline::run` starts with the input `Vec<TernaryItem>` and passes it through each stage in order. Each stage returns `Ok(next_items)` or `Err(message)`. On error, execution stops, the failing stage's name is recorded, and a `PipelineResult::err` is returned.

**Built-in stages.** `FilterStage` uses a function pointer (`fn(&TernaryItem) -> bool`) for the predicate. `TransformStage` similarly uses `fn(TernaryItem) -> TernaryItem`. This means closures that capture state are not supported—use free functions or `Pipeline::add_stage` with a custom `Box<dyn Stage>` if you need capturing closures.

**Aggregation.** `AggregateStage::sum` adds all values (clamped to i8 range) and produces a single-item result labeled "sum". `AggregateStage::count` produces the item count. Both convert to `i8`, so counts above 127 will overflow.

**Sorting.** `SortStage` sorts by the `value` field ascending (−1, 0, +1). For custom sort orders, implement `Stage` yourself.

**Builder.** `PipelineBuilder` accumulates `Box<dyn Stage>` instances and calls `Pipeline::new().add_stage()` for each. The `add_boxed` method lets you inject custom stage implementations.

## Known Limitations

- **No parallelism.** Stages execute sequentially on a single thread. There's no batched or async execution mode.
- **Aggregation overflows.** `AggregateStage::sum` clamps the total to i8 range (−127 to 127). For large inputs, the sum wraps or clips. `AggregateStage::count` has the same problem for >127 items.
- **Function pointers, not closures.** Built-in `FilterStage` and `TransformStage` use `fn` pointers, which cannot capture environment. Use `add_stage(Box::new(your_custom_stage))` for stateful stages.
- **No backpressure or streaming.** The full item vector is passed through each stage. For very large datasets, this means full materialization at every step.

## Use Cases

- **Signal processing.** Filter out neutral signals, apply transformations (negation, normalization), sort by strength, and take the top N for action.
- **Data cleanup.** Remove invalid entries (filter), normalize values (transform), and aggregate for summary statistics in one pass.
- **Strategy composition.** Chain strategy-specific stages: "remove defer signals → boost prioritize signals → limit to top 3 → aggregate for final decision."

## Ecosystem Context

Sits in the middle of the ternary stack. Consumes `TernaryItem` values that might originate from `ternary-grammar` (parsed expressions evaluated to items). Can feed into `ternary-scoring` (pipeline output as scoring candidates) or `ternary-metrics` (stages record timing data).

## License

MIT

## See Also
- **ternary-transform** — related
- **ternary-streaming** — related
- **ternary-engine** — related
- **ternary-flux** — related
- **ternary-compression** — related

