//! Composable pipelines for ternary data processing.
//!
//! Provides `Pipeline`, `Stage` trait, `PipelineBuilder`, and `PipelineResult`
//! with built-in stages for filter, transform, and aggregate operations.

use core::fmt;

/// A ternary data item flowing through the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TernaryItem {
    pub id: usize,
    pub value: i8, // -1, 0, +1
    pub label: String,
}

impl TernaryItem {
    pub fn new(id: usize, value: i8, label: &str) -> Self {
        Self { id, value, label: label.to_string() }
    }

    pub fn is_positive(&self) -> bool { self.value > 0 }
    pub fn is_negative(&self) -> bool { self.value < 0 }
    pub fn is_neutral(&self) -> bool { self.value == 0 }
}

/// Result of a pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub items: Vec<TernaryItem>,
    pub stages_run: Vec<String>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl PipelineResult {
    pub fn ok(items: Vec<TernaryItem>, stages: Vec<String>) -> Self {
        Self { items, stages_run: stages, success: true, error_message: None }
    }

    pub fn err(message: &str, stages: Vec<String>) -> Self {
        Self { items: Vec::new(), stages_run: stages, success: false, error_message: Some(message.to_string()) }
    }

    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

/// A single processing stage in a pipeline.
pub trait Stage {
    fn name(&self) -> &str;
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String>;
}

// --- Built-in stages ---

/// Filter items by a predicate.
pub struct FilterStage {
    stage_name: String,
    predicate: fn(&TernaryItem) -> bool,
}

impl FilterStage {
    pub fn new(name: &str, pred: fn(&TernaryItem) -> bool) -> Self {
        Self { stage_name: name.to_string(), predicate: pred }
    }

    pub fn positive_only() -> Self {
        Self::new("filter_positive", |item| item.is_positive())
    }

    pub fn negative_only() -> Self {
        Self::new("filter_negative", |item| item.is_negative())
    }

    pub fn non_neutral() -> Self {
        Self::new("filter_non_neutral", |item| !item.is_neutral())
    }
}

impl Stage for FilterStage {
    fn name(&self) -> &str { &self.stage_name }
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String> {
        Ok(items.into_iter().filter(self.predicate).collect())
    }
}

/// Transform items by mapping values.
pub struct TransformStage {
    stage_name: String,
    transform: fn(TernaryItem) -> TernaryItem,
}

impl TransformStage {
    pub fn new(name: &str, f: fn(TernaryItem) -> TernaryItem) -> Self {
        Self { stage_name: name.to_string(), transform: f }
    }

    pub fn negate() -> Self {
        Self::new("transform_negate", |item| TernaryItem::new(item.id, -item.value, &item.label))
    }

    pub fn set_positive() -> Self {
        Self::new("transform_positive", |item| TernaryItem::new(item.id, 1, &item.label))
    }

    pub fn relabel(_suffix: &str) -> Self {
        // Note: fn pointer can't capture; relabel is a no-op placeholder.
        // Use TransformStage::new with a custom fn for capturing behavior.
        Self::new("transform_relabel", |item| TernaryItem::new(item.id, item.value, &item.label))
    }
}

impl Stage for TransformStage {
    fn name(&self) -> &str { &self.stage_name }
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String> {
        Ok(items.into_iter().map(self.transform).collect())
    }
}

/// Aggregate items into a summary (reduces to single-item result).
pub struct AggregateStage {
    stage_name: String,
}

impl AggregateStage {
    pub fn new(name: &str) -> Self {
        Self { stage_name: name.to_string() }
    }

    pub fn sum() -> Self {
        Self::new("aggregate_sum")
    }

    pub fn count() -> Self {
        Self::new("aggregate_count")
    }
}

impl Stage for AggregateStage {
    fn name(&self) -> &str { &self.stage_name }
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String> {
        if self.stage_name == "aggregate_sum" {
            let sum: i64 = items.iter().map(|i| i.value as i64).sum();
            Ok(vec![TernaryItem::new(0, sum.clamp(-127, 127) as i8, "sum")])
        } else if self.stage_name == "aggregate_count" {
            let count = items.len() as i64;
            Ok(vec![TernaryItem::new(0, count.clamp(-127, 127) as i8, "count")])
        } else {
            Err(format!("unknown aggregate: {}", self.stage_name))
        }
    }
}

/// A stage that limits the number of items.
pub struct LimitStage {
    pub max_items: usize,
}

impl LimitStage {
    pub fn new(max: usize) -> Self { Self { max_items: max } }
}

impl Stage for LimitStage {
    fn name(&self) -> &str { "limit" }
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String> {
        Ok(items.into_iter().take(self.max_items).collect())
    }
}

/// A stage that sorts items by value.
pub struct SortStage;

impl Stage for SortStage {
    fn name(&self) -> &str { "sort" }
    fn process(&self, items: Vec<TernaryItem>) -> Result<Vec<TernaryItem>, String> {
        let mut sorted = items;
        sorted.sort_by_key(|i| i.value);
        Ok(sorted)
    }
}

/// A pipeline of stages executed in sequence.
pub struct Pipeline {
    stages: Vec<Box<dyn Stage>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(mut self, stage: Box<dyn Stage>) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn run(&self, items: Vec<TernaryItem>) -> PipelineResult {
        let mut current = items;
        let mut stages_run = Vec::new();
        for stage in &self.stages {
            match stage.process(current) {
                Ok(next) => {
                    stages_run.push(stage.name().to_string());
                    current = next;
                }
                Err(e) => {
                    stages_run.push(stage.name().to_string());
                    return PipelineResult::err(&e, stages_run);
                }
            }
        }
        PipelineResult::ok(current, stages_run)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating pipelines.
pub struct PipelineBuilder {
    stages: Vec<Box<dyn Stage>>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn filter(mut self, name: &str, pred: fn(&TernaryItem) -> bool) -> Self {
        self.stages.push(Box::new(FilterStage::new(name, pred)));
        self
    }

    pub fn transform(mut self, name: &str, f: fn(TernaryItem) -> TernaryItem) -> Self {
        self.stages.push(Box::new(TransformStage::new(name, f)));
        self
    }

    pub fn aggregate(mut self, name: &str) -> Self {
        self.stages.push(Box::new(AggregateStage::new(name)));
        self
    }

    pub fn limit(mut self, max: usize) -> Self {
        self.stages.push(Box::new(LimitStage::new(max)));
        self
    }

    pub fn sort(mut self) -> Self {
        self.stages.push(Box::new(SortStage));
        self
    }

    pub fn add_boxed(mut self, stage: Box<dyn Stage>) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn build(self) -> Pipeline {
        Pipeline { stages: self.stages }
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PipelineResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.success {
            writeln!(f, "Pipeline OK ({} stages)", self.stages_run.len())?;
            for item in &self.items {
                writeln!(f, "  [{}] {} = {}", item.id, item.label, item.value)?;
            }
        } else {
            writeln!(f, "Pipeline FAILED after {} stages: {}",
                self.stages_run.len(),
                self.error_message.as_deref().unwrap_or("unknown"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<TernaryItem> {
        vec![
            TernaryItem::new(1, 1, "pos"),
            TernaryItem::new(2, -1, "neg"),
            TernaryItem::new(3, 0, "zero"),
            TernaryItem::new(4, 1, "pos2"),
        ]
    }

    #[test]
    fn test_item_accessors() {
        let pos = TernaryItem::new(1, 1, "a");
        let neg = TernaryItem::new(2, -1, "b");
        let zero = TernaryItem::new(3, 0, "c");
        assert!(pos.is_positive());
        assert!(neg.is_negative());
        assert!(zero.is_neutral());
    }

    #[test]
    fn test_filter_positive() {
        let stage = FilterStage::positive_only();
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|i| i.is_positive()));
    }

    #[test]
    fn test_filter_negative() {
        let stage = FilterStage::negative_only();
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_non_neutral() {
        let stage = FilterStage::non_neutral();
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_transform_negate() {
        let stage = TransformStage::negate();
        let result = stage.process(vec![TernaryItem::new(1, 1, "a")]).unwrap();
        assert_eq!(result[0].value, -1);
    }

    #[test]
    fn test_transform_set_positive() {
        let stage = TransformStage::set_positive();
        let result = stage.process(vec![TernaryItem::new(1, -1, "a")]).unwrap();
        assert_eq!(result[0].value, 1);
    }

    #[test]
    fn test_aggregate_sum() {
        let stage = AggregateStage::sum();
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value, 1); // 1 + -1 + 0 + 1 = 1
    }

    #[test]
    fn test_aggregate_count() {
        let stage = AggregateStage::count();
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result[0].value, 4);
    }

    #[test]
    fn test_limit_stage() {
        let stage = LimitStage::new(2);
        let result = stage.process(sample_items()).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_sort_stage() {
        let stage = SortStage;
        let result = stage.process(sample_items()).unwrap();
        assert!(result.windows(2).all(|w| w[0].value <= w[1].value));
    }

    #[test]
    fn test_pipeline_basic() {
        let p = Pipeline::new()
            .add_stage(Box::new(FilterStage::positive_only()))
            .add_stage(Box::new(SortStage));
        let result = p.run(sample_items());
        assert!(result.success);
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.stages_run.len(), 2);
    }

    #[test]
    fn test_pipeline_empty() {
        let p = Pipeline::new();
        let result = p.run(sample_items());
        assert!(result.success);
        assert_eq!(result.items.len(), 4);
    }

    #[test]
    fn test_pipeline_builder() {
        let p = PipelineBuilder::new()
            .filter("positive", |i| i.is_positive())
            .sort()
            .limit(1)
            .build();
        let result = p.run(sample_items());
        assert!(result.success);
        assert_eq!(result.items.len(), 1);
    }

    #[test]
    fn test_pipeline_result_display() {
        let p = Pipeline::new()
            .add_stage(Box::new(FilterStage::positive_only()));
        let result = p.run(sample_items());
        let s = format!("{}", result);
        assert!(s.contains("OK"));
    }

    #[test]
    fn test_pipeline_result_error_display() {
        let result = PipelineResult::err("boom", vec!["stage1".to_string()]);
        let s = format!("{}", result);
        assert!(s.contains("FAILED"));
    }

    #[test]
    fn test_pipeline_default() {
        let p = Pipeline::default();
        assert_eq!(p.stage_count(), 0);
    }

    #[test]
    fn test_pipeline_builder_default() {
        let p = PipelineBuilder::default().build();
        assert_eq!(p.stage_count(), 0);
    }

    #[test]
    fn test_full_pipeline() {
        let p = PipelineBuilder::new()
            .filter("non_neutral", |i| !i.is_neutral())
            .transform("negate", |i| TernaryItem::new(i.id, -i.value, &i.label))
            .sort()
            .build();
        let result = p.run(sample_items());
        assert!(result.success);
        assert_eq!(result.items.len(), 3);
    }

    #[test]
    fn test_pipeline_aggregate_then_filter() {
        let p = PipelineBuilder::new()
            .aggregate("aggregate_sum")
            .build();
        let result = p.run(sample_items());
        assert!(result.success);
        assert_eq!(result.items[0].value, 1);
    }

    #[test]
    fn test_pipeline_result_empty_items() {
        let p = PipelineBuilder::new()
            .filter("positive", |i| i.is_positive())
            .build();
        let empty = vec![TernaryItem::new(1, -1, "neg"), TernaryItem::new(2, 0, "zero")];
        let result = p.run(empty);
        assert!(result.success);
        assert!(result.is_empty());
    }
}
