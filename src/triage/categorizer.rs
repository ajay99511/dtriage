use camino::Utf8Path;
use crate::config::CategorizationRule;

pub struct Categorizer {
    rules: Vec<CategorizationRule>,
}

impl Categorizer {
    pub fn new(mut rules: Vec<CategorizationRule>) -> Self {
        // Sort rules by priority (descending)
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Self { rules }
    }

    /// Categorize a file based on rules
    pub fn categorize(&self, file_path: &Utf8Path) -> Option<&CategorizationRule> {
        for rule in &self.rules {
            if rule.matches(file_path) {
                return Some(rule);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorizer_matches_pdf() {
        let rule = CategorizationRule {
            name: "Documents".to_string(),
            extensions: vec!["pdf".into()],
            destination: "Documents".to_string(),
            naming_pattern: None,
            priority: 10,
        };

        let categorizer = Categorizer::new(vec![rule]);
        let path = Utf8Path::new("/downloads/test.pdf");
        
        let result = categorizer.categorize(path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Documents");
    }

    #[test]
    fn test_categorizer_case_insensitive() {
        let rule = CategorizationRule {
            name: "Documents".to_string(),
            extensions: vec!["pdf".into()],
            destination: "Documents".to_string(),
            naming_pattern: None,
            priority: 10,
        };

        let categorizer = Categorizer::new(vec![rule]);
        let path = Utf8Path::new("/downloads/test.PDF");
        
        let result = categorizer.categorize(path);
        assert!(result.is_some());
    }

    #[test]
    fn test_categorizer_no_match() {
        let rule = CategorizationRule {
            name: "Documents".to_string(),
            extensions: vec!["pdf".into()],
            destination: "Documents".to_string(),
            naming_pattern: None,
            priority: 10,
        };

        let categorizer = Categorizer::new(vec![rule]);
        let path = Utf8Path::new("/downloads/test.xyz");
        
        let result = categorizer.categorize(path);
        assert!(result.is_none());
    }

    #[test]
    fn test_categorizer_priority_order() {
        let high_priority = CategorizationRule {
            name: "High".to_string(),
            extensions: vec!["exe".into()],
            destination: "High".to_string(),
            naming_pattern: None,
            priority: 20,
        };

        let low_priority = CategorizationRule {
            name: "Low".to_string(),
            extensions: vec!["exe".into()],
            destination: "Low".to_string(),
            naming_pattern: None,
            priority: 10,
        };

        let categorizer = Categorizer::new(vec![low_priority, high_priority]);
        let path = Utf8Path::new("/downloads/test.exe");
        
        let result = categorizer.categorize(path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "High");
    }
}
