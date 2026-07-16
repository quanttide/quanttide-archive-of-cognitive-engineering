/// 分析模块：追踪领域随时间的变化轨迹。
use crate::model::{IntentionEntry, Snapshot};
use crate::repo::Repo;

/// 返回一个领域在所有周期中的演化序列。
pub fn track_evolution(repo: &Repo, world: &str, domain: &str) -> Result<Vec<Snapshot>, String> {
    let periods = repo.periods(world)?;
    let mut snapshots = Vec::new();
    for p in &periods {
        if let Ok(file) = repo.load(world, p, domain) {
            let intentions = file.intentions.unwrap_or_default().into_iter().map(|i| IntentionEntry {
                title: i.title,
                priority: i.priority.name,
                level: i.level.name,
                risk: i.risk.name,
            }).collect();
            let entities = file.schemas.as_ref()
                .and_then(|s| s.first())
                .map(|s| s.entities.iter().map(|e| e.name.clone()).collect())
                .unwrap_or_default();
            snapshots.push(Snapshot { period: p.clone(), intentions, entities });
        }
    }
    Ok(snapshots)
}
