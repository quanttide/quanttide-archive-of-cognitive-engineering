use qtcloud_think_cli::config::Config;
use qtcloud_think_cli::repo::Repo;

fn test_repo() -> Repo {
    Repo::from_config(&Config::for_tests())
}

#[test]
fn test_worlds() {
    let repo = test_repo();
    let worlds = repo.worlds().unwrap();
    assert!(worlds.contains(&"quanttide-founder".to_string()));
}

#[test]
fn test_periods() {
    let repo = test_repo();
    let periods = repo.periods("quanttide-founder").unwrap();
    assert!(periods.contains(&"2026-W23".to_string()));
}

#[test]
fn test_domains() {
    let repo = test_repo();
    let domains = repo.domains("quanttide-founder", "2026-W23").unwrap();
    let names: Vec<&str> = domains.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"think"));
}

#[test]
fn test_load() {
    let repo = test_repo();
    let df = repo.load("quanttide-founder", "2026-W23", "think").unwrap();
    assert!(df.situations.iter().any(|js| js.situation.name == "think"));
}
