pub fn sanitize_component(input: &str) -> String {
    // Windows-unsafe characters: <>:"/\|?* plus control chars.
    // Also avoid "." and "..".
    
    // First trim whitespace
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "UNKNOWN".to_string();
    }
    
    let mut out = String::with_capacity(trimmed.len());

    for ch in trimmed.chars() {
        let ok = ch.is_ascii_alphanumeric()
            || matches!(ch, '.' | '_' | '-' );
        out.push(if ok { ch } else { '_' });
    }

    let mut s = out.trim_matches('.').to_string();
    
    // Check if result is empty or only underscores
    if s.is_empty() || s.chars().all(|c| c == '_') {
        s = "UNKNOWN".to_string();
    }

    // Avoid Windows reserved device names (minimal handling)
    let upper = s.to_ascii_uppercase();
    let reserved = ["CON","PRN","AUX","NUL","COM1","COM2","COM3","COM4","LPT1","LPT2","LPT3"];
    if reserved.contains(&upper.as_str()) {
        s = format!("_{}", s);
    }

    // Keep it reasonable
    s.chars().take(80).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_alphanumeric() {
        assert_eq!(sanitize_component("abc123"), "abc123");
        assert_eq!(sanitize_component("Test_File-01.dcm"), "Test_File-01.dcm");
    }

    #[test]
    fn test_windows_unsafe_chars() {
        assert_eq!(sanitize_component("file<>name"), "file__name");
        assert_eq!(sanitize_component("path/to\\file"), "path_to_file");
        assert_eq!(sanitize_component("file:name|test"), "file_name_test");
        assert_eq!(sanitize_component("file?name*"), "file_name_");
        assert_eq!(sanitize_component("file\"name"), "file_name");
    }

    #[test]
    fn test_dots() {
        assert_eq!(sanitize_component("..."), "UNKNOWN");
        assert_eq!(sanitize_component(".hidden"), "hidden");
        assert_eq!(sanitize_component("file.txt."), "file.txt");
        assert_eq!(sanitize_component(".."), "UNKNOWN");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(sanitize_component(""), "UNKNOWN");
        assert_eq!(sanitize_component("   "), "UNKNOWN");
    }

    #[test]
    fn test_reserved_names() {
        assert_eq!(sanitize_component("CON"), "_CON");
        assert_eq!(sanitize_component("con"), "_con");
        assert_eq!(sanitize_component("PRN"), "_PRN");
        assert_eq!(sanitize_component("AUX"), "_AUX");
        assert_eq!(sanitize_component("NUL"), "_NUL");
        assert_eq!(sanitize_component("COM1"), "_COM1");
        assert_eq!(sanitize_component("LPT1"), "_LPT1");
        
        // Non-reserved should pass through
        assert_eq!(sanitize_component("CONSOLE"), "CONSOLE");
        assert_eq!(sanitize_component("COM10"), "COM10");
    }

    #[test]
    fn test_length_limit() {
        let long = "a".repeat(100);
        let result = sanitize_component(&long);
        assert_eq!(result.len(), 80);
        assert_eq!(result, "a".repeat(80));
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(
            sanitize_component("Patient_001<DOE>JOHN:2026"),
            "Patient_001_DOE_JOHN_2026"
        );
    }

    #[test]
    fn test_unicode() {
        // Unicode characters should be replaced with underscore
        // Pure unicode becomes UNKNOWN
        assert_eq!(sanitize_component("文件名"), "UNKNOWN");
        // Mixed content keeps the ASCII part (名称 = 2 chars = 2 underscores)
        assert_eq!(sanitize_component("file_名称"), "file___");
    }
}
