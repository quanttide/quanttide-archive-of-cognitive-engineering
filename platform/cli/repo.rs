use std::path::{Path, PathBuf};

use crate::model::IntentionQueryResult;

use quanttide_think::{
    domain::Domain,
    intention::Intention,
    schema::SchemaContent,
    situation::Situation,
    situation_relation::SituationRelation,
};

#[derive(Debug)]
pub struct Repo {
    path: PathBuf,
}

impl Repo {
    pub fn from_config(cfg: &crate::config::Config) -> Self {
        Self::open(&cfg.journal_path)
    }
    /// Open a journal at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        Self { path: path.as_ref().to_path_buf() }
    }

    /// List all worlds (directories under root).
    pub fn worlds(&self) -> Result<Vec<String>, String> {
        let mut worlds = Vec::new();
        let entries = std::fs::read_dir(&self.path)
            .map_err(|e| format!("cannot read journal: {}", e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    worlds.push(name.to_string());
                }
            }
        }
        worlds.sort();
        Ok(worlds)
    }

    /// List all periods (weeks) for a world.
    pub fn periods(&self, world: &str) -> Result<Vec<String>, String> {
        let dir = self.path.join(world);
        let mut periods = Vec::new();
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("cannot read world {}: {}", world, e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    periods.push(name.to_string());
                }
            }
        }
        periods.sort();
        Ok(periods)
    }

    /// List all domains for a world + period.
    pub fn domains(&self, world: &str, period: &str) -> Result<Vec<Domain>, String> {
        let dir = self.path.join(world).join(period);
        let mut domains = Vec::new();
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| format!("cannot read {}/{}: {}", world, period, e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yaml") {
                if let Some(stem) = path.file_stem().and_then(|n| n.to_str()) {
                    if stem != "thoughts" {
                        let label = stem.to_string();
                        domains.push(Domain { name: stem.to_string(), label });
                    }
                }
            }
        }
        domains.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(domains)
    }

    /// Load a domain file.
    pub fn load(&self, world: &str, period: &str, domain: &str) -> Result<DomainFile, String> {
        let path = self.path.join(world).join(period).join(format!("{}.yaml", domain));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        let raw: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("cannot parse {}: {}", path.display(), e))?;

        let schemas = raw.get("schemas")
            .and_then(|v| serde_yaml::from_value::<Vec<SchemaContent>>(v.clone()).ok());

        let situations = raw.get("situations")
            .and_then(|arr| arr.as_sequence())
            .map(|seq| seq.iter().filter_map(JournalSituation::from_value).collect())
            .unwrap_or_default();

        let intentions = raw.get("intentions")
            .and_then(|v| serde_yaml::from_value::<Vec<Intention>>(v.clone()).ok());

        let thoughts = raw.get("thoughts")
            .and_then(|v| serde_yaml::from_value::<Vec<String>>(v.clone()).ok());

        Ok(DomainFile {
            schemas,
            situations,
            intentions,
            thoughts,
        })
    }

    /// 查询指定 world/period/domain 的意向列表。
    pub fn intentions(&self, world: &str, period: &str, domain: &str) -> Result<Vec<Intention>, String> {
        let file = self.load(world, period, domain)?;
        Ok(file.intentions.unwrap_or_default())
    }

    /// 多条件过滤查询某 world 下所有 period/domain 的意向。
    pub fn all_intentions(
        &self,
        world: &str,
        priority: Option<&str>,
        risk: Option<&str>,
        level: Option<&str>,
    ) -> Result<Vec<IntentionQueryResult>, String> {
        let mut results = Vec::new();
        let periods = self.periods(world)?;
        for p in &periods {
            let domains = self.domains(world, p)?;
            for d in &domains {
                if let Ok(file) = self.load(world, p, &d.name) {
                    if let Some(intents) = file.intentions {
                        for i in intents {
                            if let Some(p_val) = priority {
                                if i.priority.name != p_val { continue; }
                            }
                            if let Some(r_val) = risk {
                                if i.risk.name != r_val { continue; }
                            }
                            if let Some(l_val) = level {
                                if i.level.name != l_val { continue; }
                            }
                            results.push(IntentionQueryResult {
                                world: world.to_string(),
                                period: p.clone(),
                                domain: d.name.clone(),
                                intention: i,
                            });
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// 按 UUID 查询意向详情。
    pub fn intention_by_id(&self, world: &str, id: &str) -> Result<Option<IntentionQueryResult>, String> {
        let periods = self.periods(world)?;
        for p in &periods {
            let domains = self.domains(world, p)?;
            for d in &domains {
                if let Ok(file) = self.load(world, p, &d.name) {
                    if let Some(intents) = file.intentions {
                        for i in intents {
                            if i.id.to_string() == id {
                                return Ok(Some(IntentionQueryResult {
                                    world: world.to_string(),
                                    period: p.clone(),
                                    domain: d.name.clone(),
                                    intention: i,
                                }));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// 描述某周各领域的数据情况。
    pub fn describe(&self, world: &str, period: &str) -> Result<Vec<DataCoherence>, String> {
        let domains = self.domains(world, period)?;
        let mut results = Vec::new();
        for d in &domains {
            let file = self.load(world, period, &d.name)?;
            results.push(DataCoherence {
                domain: d.name.clone(),
                intentions: file.intentions.as_ref().map(|v| v.len()).unwrap_or(0),
                schemas: file.schemas.is_some(),
                relations: file.relations().len(),
            });
        }
        Ok(results)
    }
}

#[derive(Debug)]
pub struct DataCoherence {
    pub domain: String,
    pub intentions: usize,
    pub schemas: bool,
    pub relations: usize,
}

#[derive(Debug, Clone)]
pub struct JournalSituation {
    pub situation: Situation,
    pub relations: Vec<SituationRelation>,
}

impl JournalSituation {
    pub fn from_value(v: &serde_yaml::Value) -> Option<Self> {
        let situation: Situation = serde_yaml::from_value(v.clone()).ok()?;
        let relations = v.get("relations")
            .and_then(|r| serde_yaml::from_value(r.clone()).ok())
            .unwrap_or_default();
        Some(Self { situation, relations })
    }
}

#[derive(Debug)]
pub struct DomainFile {
    pub schemas: Option<Vec<SchemaContent>>,
    pub situations: Vec<JournalSituation>,
    pub intentions: Option<Vec<Intention>>,
    pub thoughts: Option<Vec<String>>,
}

impl DomainFile {
    /// The situation name is the primary identifier.
    pub fn situation(&self) -> Option<&Situation> {
        self.situations.first().map(|js| &js.situation)
    }

    /// All relations from the embedded situation entries.
    pub fn relations(&self) -> Vec<&SituationRelation> {
        self.situations.iter().flat_map(|js| js.relations.iter()).collect()
    }
}
