use quanttide_think::intention::Intention;

/// 一个领域在某周的快照。
#[derive(Debug)]
pub struct Snapshot {
    pub period: String,
    pub intentions: Vec<IntentionEntry>,
    pub entities: Vec<String>,
}

/// 快照中的一条意图记录。
#[derive(Debug)]
pub struct IntentionEntry {
    pub title: String,
    pub priority: String,
    pub level: String,
    pub risk: String,
}

/// 意向查询结果，包含上下文信息。
#[derive(Debug)]
pub struct IntentionQueryResult {
    pub world: String,
    pub period: String,
    pub domain: String,
    pub intention: Intention,
}
