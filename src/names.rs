use rand::seq::SliceRandom;

const ADJECTIVES: &[&str] = &[
    "amber", "bold", "bright", "calm", "clever",
    "cool", "crisp", "daring", "eager", "fair",
    "fast", "fierce", "gentle", "glad", "golden",
    "grand", "happy", "hardy", "keen", "kind",
    "light", "lively", "lucky", "merry", "mighty",
    "noble", "pale", "proud", "quick", "quiet",
    "rapid", "ready", "rosy", "sharp", "shy",
    "sleek", "slim", "smart", "soft", "steady",
    "still", "stout", "strong", "sunny", "sure",
    "sweet", "swift", "tall", "warm", "wise",
];

const ANIMALS: &[&str] = &[
    "ant", "bat", "bear", "bee", "bird",
    "buck", "bull", "cat", "colt", "crab",
    "crow", "deer", "doe", "dove", "duck",
    "elk", "fawn", "fish", "frog", "goat",
    "hare", "hawk", "jay", "lark", "lion",
    "lynx", "mole", "moth", "newt", "orca",
    "owl", "puma", "ram", "rat", "seal",
    "slug", "snail", "swan", "toad", "vole",
    "wasp", "whale", "wolf", "wren", "yak",
    "fox", "ape", "asp", "cod", "emu",
];

/// Generate a random adjective-animal name like "swift-fox"
pub fn generate_name() -> String {
    let mut rng = rand::thread_rng();
    let adj = ADJECTIVES.choose(&mut rng).unwrap();
    let animal = ANIMALS.choose(&mut rng).unwrap();
    format!("{}-{}", adj, animal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn name_format_is_adjective_dash_animal() {
        for _ in 0..100 {
            let name = generate_name();
            let parts: Vec<&str> = name.split('-').collect();
            assert_eq!(parts.len(), 2, "Name should have exactly one dash: {}", name);
            assert!(ADJECTIVES.contains(&parts[0]), "Bad adjective: {}", parts[0]);
            assert!(ANIMALS.contains(&parts[1]), "Bad animal: {}", parts[1]);
        }
    }

    #[test]
    fn names_have_variety() {
        let names: HashSet<String> = (0..100).map(|_| generate_name()).collect();
        assert!(names.len() > 10, "Expected variety, got {} unique names", names.len());
    }

    #[test]
    fn word_list_sizes() {
        assert_eq!(ADJECTIVES.len(), 50);
        assert_eq!(ANIMALS.len(), 50);
    }
}
