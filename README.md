# ternary-pipeline

Composable data processing pipelines for ternary {-1, 0, +1} items. Builder-pattern pipeline construction with filter, transform, aggregate, sort, and limit stages — plus a `Stage` trait for custom processing logic.

## Why It Matters

Ternary data processing often requires multi-step transformations: filter out neutral values, negate signals, aggregate by sum or count, sort by value, and limit results. Hard-coding these sequences is fragile and non-reusable. This crate provides:

- **Composable stages**: chain filter → transform → aggregate in any order
- **Builder pattern**: fluent `PipelineBuilder::new().filter(...).sort().limit(n).build()`
- **Error propagation**: any stage can fail, halting the pipeline with a diagnostic message
- **Custom stages**: implement the `Stage` trait for domain-specific processing
- **Zero dependencies**: pure Rust, no external crates

Applications include ternary signal conditioning, agent decision post-processing, multi-stage feature extraction, and stream processing of ternary event feeds.

## How It Works

### Pipeline Model

A pipeline is an ordered sequence of stages. Each stage takes `Vec<TernaryItem>` and returns `Result<Vec<TernaryItem>, String>`:

$$\text{Pipeline}(\mathbf{x}) = S_n(\ldots S_2(S_1(\mathbf{x})) \ldots)$$

Execution is eager: each stage runs to completion before the next begins.

### TernaryItem

Each item carries:
- `id: usize` — unique identifier
- `value: i8` — the ternary value ∈ {-1, 0, +1}
- `label: String` — human-readable annotation

### Stage Trait

```rust
pub trait Stage {
    fn name(&self) -> &str;
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String>;
}
```

### Built-in Stages

| Stage | Operation | Complexity |
|---|---|---|
| `FilterStage` | Keep items matching predicate | O($n$) |
| `TransformStage` | Map each item to new item | O($n$) |
| `AggregateStage::sum()` | Reduce to single item: $\sum_i v_i$ | O($n$) |
| `AggregateStage::count()` | Reduce to single item: $n$ | O($n$) |
| `SortStage` | Sort by value ascending | O($n \log n$) |
| `LimitStage` | Keep first $k$ items | O($k$) |

### Error Handling

If any stage returns `Err(msg)`, the pipeline halts immediately and returns a `PipelineResult` with `success = false` and the error message. The stages that ran successfully are recorded for debugging.

### Composition

Pipelines are built fluently:

```rust
let p = PipelineBuilder::new()
    .filter("positive", |i| i.is_positive())
    .transform("negate", |i| TernaryItem::new(i.id, -i.value, &i.label))
    .sort()
    .limit(10)
    .build();
```

**Total complexity:** sum of individual stage complexities, since stages execute sequentially.

## Quick Start

```rust
use ternary_pipeline::*;

fn sample() -> Vec<TernaryItem> {
    vec![
        TernaryItem::new(1, 1, "accept"),
        TernaryItem::new(2, -1, "reject"),
        TernaryItem::new(3, 0, "abstain"),
        TernaryItem::new(4, 1, "accept2"),
    ]
}

// Filter + sort
let p = Pipeline::new()
    .add_stage(Box::new(FilterStage::positive_only()))
    .add_stage(Box::new(SortStage));
let result = p.run(sample());
assert!(result.success);
assert_eq!(result.items.len(), 2);
assert_eq!(result.stages_run, vec!["filter_positive", "sort"]);

// Builder with aggregate
let p = PipelineBuilder::new()
    .filter("non_neutral", |i| !i.is_neutral())
    .aggregate("aggregate_sum")
    .build();
let result = p.run(sample());
assert_eq!(result.items[0].value, 1); // 1 + (-1) + 1 = 1

// Display output
let p = PipelineBuilder::new()
    .filter("positive", |i| i.is_positive())
    .build();
let result = p.run(sample());
println!("{}", result);
// Pipeline OK (1 stages)
//   [1] accept = 1
//   [4] accept2 = 1
```

## API

| Type | Description |
|---|---|
| `TernaryItem::new(id, value, label)` | Create a ternary data item |
| `.is_positive() / .is_negative() / .is_neutral()` | Value predicates |
| `Pipeline::new()` | Empty pipeline |
| `.add_stage(boxed) → Self` | Add a stage (builder-style) |
| `.run(items) → PipelineResult` | Execute pipeline |
| `PipelineBuilder::new()` | Fluent builder |
| `.filter(name, pred) / .transform(name, f)` | Built-in stages |
| `.aggregate(name) / .sort() / .limit(n)` | More built-in stages |
| `.add_boxed(boxed) / .build()` | Custom stages and build |
| `PipelineResult` | Result with `items`, `stages_run`, `success`, `error_message` |
| `Stage` trait | Implement for custom stages |

## Architecture Notes

The pipeline architecture reflects the **γ + η = C** conservation identity in its data flow. Each `TernaryItem` carries a ternary value: +1 contributes to the constructive mass γ, -1 to the inhibitory mass η, and 0 to the neutral pool. The total count $C = |\text{items}|$ is only reduced by explicit filter or limit stages — transform and aggregate stages preserve the *information content* of the stream even as they reshape its representation.

The filter stage acts as a conservation gate: it selects which subset of {γ, η, neutral} populations pass through, modifying the effective $C$ for downstream stages. The aggregate stage performs the summation $\sum_i v_i$, directly computing $\gamma - \eta$ (the net constructive-inhibitory balance) over the input population. This makes aggregation the point where the conservation identity becomes numerically observable.

## References

- Gamma, E. et al. (1994). *Design Patterns.* Addison-Wesley. (Builder, Chain of Responsibility)
- Alexandrescu, A. (2001). *Modern C++ Design.* Addison-Wesley. (Policy-based design)
- Okken, P. (2022). *Python Testing with pytest.* 2nd ed. (Pipeline testing patterns)
- Newman, S. (2021). *Building Microservices.* 2nd ed. O'Reilly. (Stream processing)

## License

MIT
