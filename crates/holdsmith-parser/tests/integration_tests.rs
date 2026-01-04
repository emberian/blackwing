use holdsmith_parser::{parse, Context, Effect, PassageContent};
use std::fs;

const SIMPLE_SCENE: &str = include_str!("../../../tools/holdsmith/tests/fixtures/valid/simple.scene");
const BRANCHING_SCENE: &str = include_str!("../../../tools/holdsmith/tests/fixtures/valid/branching.scene");
const SERA_SCENE: &str = include_str!("../../../src/content/scenelets/scenes/journey/sera_contact.scene");

#[test]
fn test_parse_simple_fixture() {
    let result = parse(SIMPLE_SCENE, "simple.scene");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let scene = result.unwrap();
    assert_eq!(scene.frontmatter.id.as_str(), "simple_test");
    assert_eq!(scene.frontmatter.title.as_str(), "Simple Test Scene");
    assert_eq!(scene.frontmatter.context, Context::Journey);
    assert_eq!(scene.frontmatter.weight, 10);
    assert_eq!(scene.frontmatter.cooldown, 5);
    assert!(scene.frontmatter.tags.contains(&"test".into()));
    assert_eq!(scene.passages.len(), 1);
    assert_eq!(scene.passages[0].name.as_str(), "intro");
}

#[test]
fn test_parse_branching_fixture() {
    let result = parse(BRANCHING_SCENE, "branching.scene");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let scene = result.unwrap();
    assert_eq!(scene.frontmatter.id.as_str(), "branching_test");
    assert_eq!(scene.frontmatter.context, Context::Port);
    assert!(scene.frontmatter.requires.is_some());

    let requires = scene.frontmatter.requires.as_ref().unwrap();
    assert!(requires.ship_tags.contains(&"sensor".into()));
    assert_eq!(requires.min_resources.len(), 1);
    assert_eq!(requires.min_resources[0].resource.as_str(), "credits");
    assert_eq!(requires.min_resources[0].value, 50);

    assert_eq!(scene.passages.len(), 3);
    assert_eq!(scene.passages[0].name.as_str(), "intro");
    assert_eq!(scene.passages[1].name.as_str(), "left_path");
    assert_eq!(scene.passages[2].name.as_str(), "right_path");
}

#[test]
fn test_parse_sera_contact() {
    let result = parse(SERA_SCENE, "sera_contact.scene");
    assert!(result.is_ok(), "Parse failed: {:?}", result.err());

    let scene = result.unwrap();
    assert_eq!(scene.frontmatter.id.as_str(), "journey_sera_contact");
    assert_eq!(scene.frontmatter.title.as_str(), "Sera Infestation");
    assert!(scene.frontmatter.tags.contains(&"danger".into()));
    assert!(scene.frontmatter.tags.contains(&"sera".into()));

    // Should have 4 passages: intro, vent, contain, assess
    assert_eq!(scene.passages.len(), 4);
    assert_eq!(scene.passages[0].name.as_str(), "intro");
    assert_eq!(scene.passages[1].name.as_str(), "vent");
    assert_eq!(scene.passages[2].name.as_str(), "contain");
    assert_eq!(scene.passages[3].name.as_str(), "assess");

    // Check the intro passage has 3 choices
    let intro = &scene.passages[0];
    let choices: Vec<_> = intro
        .content
        .iter()
        .filter_map(|c| match c {
            PassageContent::Choice(choice) => Some(choice),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 3);

    // Check the vent passage has effects
    let vent = &scene.passages[1];
    let vent_choices: Vec<_> = vent
        .content
        .iter()
        .filter_map(|c| match c {
            PassageContent::Choice(choice) => Some(choice),
            _ => None,
        })
        .collect();
    assert_eq!(vent_choices.len(), 1);

    let vent_choice = &vent_choices[0];
    assert!(vent_choice.effects.len() >= 2);

    // Should have removeCards effect
    let has_remove_cards = vent_choice.effects.iter().any(|e| {
        matches!(e, Effect::RemoveCards(rc) if rc.pattern.contains("cargo"))
    });
    assert!(has_remove_cards, "Expected removeCards effect");

    // Should have flag effect
    let has_flag = vent_choice.effects.iter().any(|e| {
        matches!(e, Effect::Flag(f) if f.flag.as_str() == "sera_survived")
    });
    assert!(has_flag, "Expected flag effect");
}

#[test]
fn test_parse_all_scene_files() {
    // Get the workspace root (crates/holdsmith-parser -> workspace root)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Journey scenes
    let journey_dir = workspace_root.join("src/content/scenelets/scenes/journey");
    let port_dir = workspace_root.join("src/content/scenelets/scenes/port");

    let mut total = 0;
    let mut passed = 0;
    let mut failures = Vec::new();

    for dir in [&journey_dir, &port_dir] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "scene") {
                    total += 1;
                    let content = fs::read_to_string(&path).expect("Failed to read file");
                    let filename = path.file_name().unwrap().to_string_lossy();

                    match parse(&content, &filename) {
                        Ok(_) => {
                            passed += 1;
                        }
                        Err(e) => {
                            failures.push(format!("{}: {:?}", filename, e));
                        }
                    }
                }
            }
        }
    }

    println!("Parsed {}/{} scene files successfully", passed, total);
    if !failures.is_empty() {
        eprintln!("Failures:\n{}", failures.join("\n"));
    }

    assert!(
        total > 0,
        "Should have found at least one scene file. Checked dirs: {:?}, {:?}",
        journey_dir,
        port_dir
    );
    assert_eq!(
        passed, total,
        "All scene files should parse successfully"
    );
}
