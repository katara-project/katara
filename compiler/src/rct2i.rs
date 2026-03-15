//! # RCT2I Prompt Compiler
//!
//! Restructures unstructured prompts into the **RCT2I** framework
//! (Role, Context, Tasks, Instructions, Improvement) for more efficient
//! LLM consumption.
//!
//! ## How it saves tokens
//!
//! 1. Deduplicates scattered intent/role signals into a single Role line.
//! 2. Separates context from instructions — context can be compressed harder.
//! 3. Extracts actionable tasks into a compact list.
//! 4. Removes conversational filler that doesn't contribute to any section.
//!
//! Applied *after* the BPE optimizer and *before* the convergence loop.

/// RCT2I-structured output.
pub struct Rct2iResult {
    pub structured: String,
    pub sections_found: u8,
}

/// Attempt to restructure the prompt into RCT2I format.
///
/// Only restructures if we detect at least a task component (otherwise the
/// prompt is too short/ambiguous to benefit).  Returns None when the input
/// doesn't warrant restructuring (≤30 tokens or already structured).
pub fn restructure(text: &str, intent: &str) -> Option<Rct2iResult> {
    let trimmed = text.trim();
    // V10.16: lowered from 12→4 words so most real requests get restructured.
    if trimmed.is_empty() || trimmed.split_whitespace().count() <= 4 {
        return None;
    }

    // Don't restructure OCR or translate — content must be preserved verbatim.
    if matches!(intent, "ocr" | "translate") {
        return None;
    }

    // Skip if the input is a raw technical artifact (stack traces, diffs,
    // compiler output) — flattening these into RCT2I sections would destroy
    // the line-level structure that semantic reducers rely on.
    if is_raw_artifact(trimmed) {
        return None;
    }

    // Already structured? (starts with RCT2I markers or markdown headers)
    if looks_structured(trimmed) {
        return None;
    }

    let role = extract_role(trimmed, intent);
    let (context_lines, task_lines, instruction_lines) = classify_lines(trimmed);

    // Must have at least tasks to justify restructuring.
    if task_lines.is_empty() {
        return None;
    }

    let mut out = String::with_capacity(trimmed.len());
    let mut sections: u8 = 0;

    // R — Role (always emit, inferred from intent if not explicit)
    out.push_str("[R] ");
    out.push_str(&role);
    out.push('\n');
    sections += 1;

    // C — Context (if any)
    if !context_lines.is_empty() {
        out.push_str("[C] ");
        out.push_str(&context_lines.join(" "));
        out.push('\n');
        sections += 1;
    }

    // T — Tasks
    out.push_str("[T] ");
    out.push_str(&task_lines.join("; "));
    out.push('\n');
    sections += 1;

    // I — Instructions (constraints, format requirements)
    if !instruction_lines.is_empty() {
        out.push_str("[I] ");
        out.push_str(&instruction_lines.join("; "));
        out.push('\n');
        sections += 1;
    }

    // I — Improvement (auto-generated quality hint based on intent)
    let improv = improvement_hint(intent);
    if !improv.is_empty() {
        out.push_str("[I] ");
        out.push_str(improv);
        out.push('\n');
        sections += 1;
    }

    Some(Rct2iResult {
        structured: out.trim_end().to_string(),
        sections_found: sections,
    })
}

fn looks_structured(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Check for existing RCT2I markers or common structured formats.
    lower.starts_with("[r]")
        || lower.starts_with("[c]")
        || lower.starts_with("[t]")
        || lower.starts_with("role:")
        || lower.starts_with("context:")
        || lower.starts_with("task:")
        || lower.starts_with("# role")
        || lower.starts_with("## role")
}

fn extract_role(text: &str, intent: &str) -> String {
    let lower = text.to_lowercase();

    // Try to find explicit role signals.
    let role_phrases = [
        "you are a",
        "you are an",
        "act as a",
        "act as an",
        "as a",
        "as an",
        "tu es un",
        "tu es une",
        "agis comme",
        "sois un",
        "sois une",
    ];

    for phrase in &role_phrases {
        if let Some(pos) = lower.find(phrase) {
            let after = &text[pos + phrase.len()..];
            let end = after
                .find(['.', ',', '\n', ';'])
                .unwrap_or(after.len().min(60));
            let role = after[..end].trim();
            if !role.is_empty() {
                return role.to_string();
            }
        }
    }

    // Infer from intent.
    match intent {
        "debug" => "debugging assistant".to_string(),
        "review" => "code reviewer".to_string(),
        "codegen" => "code generator".to_string(),
        "summarize" | "fast" => "summarization assistant".to_string(),
        "quality" => "senior engineering assistant".to_string(),
        _ => "general assistant".to_string(),
    }
}

fn classify_lines(text: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut context = Vec::new();
    let mut tasks = Vec::new();
    let mut instructions = Vec::new();

    // Split into segments: use lines first, then sentences within long lines.
    let segments = split_into_segments(text);

    for segment in &segments {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();

        // Skip role-declaration lines (already extracted).
        if is_role_line(&lower) {
            continue;
        }

        if is_instruction_line(&lower) {
            instructions.push(trimmed.to_string());
        } else if is_task_line(&lower) {
            tasks.push(trimmed.to_string());
        } else {
            context.push(trimmed.to_string());
        }
    }

    // If no explicit tasks found, promote content to tasks so RCT2I always
    // produces a [T] section for any non-trivial input.
    if tasks.is_empty() && !context.is_empty() {
        tasks.push(context.remove(0));
    }
    // V10.16: last resort — collapse remaining context into a single task
    // rather than returning None.  Every request deserves structuring.
    if tasks.is_empty() && !instructions.is_empty() {
        tasks.push(instructions.remove(0));
    }

    (context, tasks, instructions)
}

/// Split text into classifiable segments. If the text has multiple lines, use
/// lines. If it's a single long line, split on sentence boundaries (`. `).
fn split_into_segments(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() > 1 {
        return lines.iter().map(|l| l.to_string()).collect();
    }
    // Single line — split on sentence boundaries.
    let line = lines.first().copied().unwrap_or("");
    line.split(". ")
        .map(|s| {
            let t = s.trim();
            if t.ends_with('.') {
                t.to_string()
            } else {
                format!("{t}.")
            }
        })
        .filter(|s| s.len() > 1)
        .collect()
}

fn is_role_line(lower: &str) -> bool {
    lower.starts_with("you are")
        || lower.starts_with("act as")
        || lower.starts_with("tu es")
        || lower.starts_with("agis comme")
        || lower.starts_with("sois un")
        || lower.starts_with("sois une")
}

fn is_task_line(lower: &str) -> bool {
    lower.starts_with("- ")
        || lower.starts_with("* ")
        || lower.starts_with("1.")
        || lower.starts_with("2.")
        || lower.starts_with("3.")
        // EN task verbs
        || lower.contains("implement")
        || lower.contains("create")
        || lower.contains("write")
        || lower.contains("fix")
        || lower.contains("add")
        || lower.contains("remove")
        || lower.contains("update")
        || lower.contains("refactor")
        || lower.contains("build")
        || lower.contains("explain")
        || lower.contains("show")
        || lower.contains("analyze")
        || lower.contains("analyse")
        || lower.contains("find")
        || lower.contains("check")
        || lower.contains("test")
        || lower.contains("describe")
        || lower.contains("compare")
        || lower.contains("convert")
        || lower.contains("migrate")
        || lower.contains("deploy")
        || lower.contains("install")
        || lower.contains("configure")
        || lower.contains("optimize")
        || lower.contains("improve")
        || lower.contains("change")
        || lower.contains("modify")
        || lower.contains("debug")
        || lower.contains("trace")
        || lower.contains("review")
        || lower.contains("summarize")
        || lower.contains("translate")
        || lower.contains("generate")
        || lower.contains("design")
        || lower.contains("delete")
        || lower.contains("move")
        || lower.contains("rename")
        || lower.contains("help")
        // FR task verbs
        || lower.contains("implémente")
        || lower.contains("crée")
        || lower.contains("écris")
        || lower.contains("corrige")
        || lower.contains("ajoute")
        || lower.contains("supprime")
        || lower.contains("explique")
        || lower.contains("montre")
        || lower.contains("analyse")
        || lower.contains("vérifie")
        || lower.contains("teste")
        || lower.contains("décris")
        || lower.contains("trouve")
        || lower.contains("cherche")
        || lower.contains("résous")
        || lower.contains("optimise")
        || lower.contains("améliore")
        || lower.contains("modifie")
        || lower.contains("configure")
        || lower.contains("installe")
        || lower.contains("déploie")
        || lower.contains("génère")
        || lower.contains("conçois")
        || lower.contains("renomme")
        || lower.contains("déplace")
        || lower.contains("aide")
        || lower.contains("répare")
        || lower.contains("mets à jour")
        || lower.contains("fais")
        || lower.contains("fait")
}

fn is_instruction_line(lower: &str) -> bool {
    lower.contains("must ")
        || lower.contains("should ")
        || lower.contains("don't ")
        || lower.contains("do not ")
        || lower.contains("make sure")
        || lower.contains("ensure")
        || lower.contains("format ")
        || lower.contains("use ")
        || lower.contains("avoid ")
        || lower.contains("keep ")
        || lower.contains("return ")
        || lower.contains("output ")
        || lower.contains("only ")
        || lower.contains("always ")
        || lower.contains("never ")
        || lower.contains("prefer ")
        || lower.contains("instead ")
        || lower.contains("without ")
        || lower.contains("in json")
        || lower.contains("in typescript")
        || lower.contains("in rust")
        || lower.contains("in python")
        || lower.contains("in go")
        || lower.contains("in kotlin")
        || lower.contains("in java")
        // FR instruction markers
        || lower.contains("doit ")
        || lower.contains("ne pas ")
        || lower.contains("assure-toi")
        || lower.contains("utilise ")
        || lower.contains("évite ")
        || lower.contains("il faut")
        || lower.contains("sans ")
        || lower.contains("toujours ")
        || lower.contains("jamais ")
        || lower.contains("plutôt ")
        || lower.contains("seulement ")
        || lower.contains("en json")
        || lower.contains("en typescript")
        || lower.contains("en rust")
        || lower.contains("en python")
}

/// Detect raw technical artifacts (stack traces, diffs, compiler output) that
/// should NOT be restructured — their line structure is meaningful.
fn is_raw_artifact(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 2 {
        return false;
    }
    let artifact_count = lines
        .iter()
        .filter(|l| {
            let t = l.trim().to_lowercase();
            t.starts_with("trace:")
                || t.starts_with("at ")
                || t.starts_with("panic:")
                || t.starts_with("error:")
                || t.starts_with("warning:")
                || t.starts_with("-->")
                || t.starts_with("diff ")
                || t.starts_with("@@")
                || t.starts_with("+ ")
                || t.starts_with("- ")
                || t.starts_with("info ")
                || t.starts_with("| ")
                || t.starts_with("fatal:")
                || t.starts_with("caused by:")
                || t.starts_with("note:")
                || t == "|"
        })
        .count();
    // If more than half of lines are artifact markers, skip RCT2I
    artifact_count * 2 > lines.len()
}

fn improvement_hint(intent: &str) -> &'static str {
    match intent {
        "debug" => "verify fix correctness, suggest root cause",
        "review" => "check for bugs, performance, security",
        "codegen" => "write idiomatic, tested, minimal code",
        "summarize" | "fast" => "be concise, preserve key facts",
        "quality" => "thorough analysis, best practices",
        _ => "be clear, direct, and actionable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_prompt_returns_none() {
        // 3 words: below the 4-word threshold
        assert!(restructure("fix the bug", "debug").is_none());
    }

    #[test]
    fn ocr_intent_returns_none() {
        let long = "Extract all text from this image. The image contains a receipt with multiple items and prices listed.";
        assert!(restructure(long, "ocr").is_none());
    }

    #[test]
    fn already_structured_returns_none() {
        let text = "[R] code reviewer\n[C] reviewing a PR\n[T] find bugs";
        assert!(restructure(text, "review").is_none());
    }

    #[test]
    fn debug_intent_gets_restructured() {
        let prompt = "error: mismatched types at line 42 in main.rs. The function expects i32 but received a string. Please explain what is wrong.";
        let result = restructure(prompt, "debug");
        assert!(result.is_some(), "debug prompts should be restructured");
        let r = result.unwrap();
        assert!(r.structured.contains("[R]"));
        assert!(r.structured.contains("[I]"));
        assert!(r.structured.contains("root cause"));
    }

    #[test]
    fn codegen_prompt_restructured() {
        let prompt = "You are a Rust developer. We have a web server using Actix. \
                       Implement a new endpoint that returns health status. \
                       Use JSON format. Make sure to add tests.";
        let result = restructure(prompt, "codegen");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.structured.contains("[R]"));
        assert!(r.structured.contains("[T]"));
        assert!(r.sections_found >= 3);
    }

    #[test]
    fn explicit_role_extracted() {
        let prompt = "You are a security expert. Review this code for SQL injection vulnerabilities. \
                       The code uses raw string concatenation for database queries. \
                       Make sure to check all user input paths. Ensure parameterized queries are used.";
        let result = restructure(prompt, "review");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.structured.contains("security expert"));
    }

    #[test]
    fn intent_inferred_role() {
        let prompt =
            "Review the code in my Rust application at line 42 of main.rs. The diff shows \
                       a removed unwrap on a None value. Refactor the code to handle the Option properly. \
                       Add error handling with anyhow.";
        let result = restructure(prompt, "review");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.structured.contains("code reviewer"));
    }

    #[test]
    fn improvement_added_for_codegen() {
        let prompt = "We need a function to parse CSV files into structs. The CSV has headers name, age, email. \
                       Implement the parser in Rust. Use serde for deserialization.";
        let result = restructure(prompt, "codegen");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.structured.contains("[I]"));
        assert!(r.structured.contains("idiomatic"));
    }

    #[test]
    fn french_prompt_restructured() {
        let prompt = "Tu es un développeur senior. Nous avons une API REST en Go. \
                       Implémente un middleware d'authentification JWT. \
                       Utilise la bibliothèque golang-jwt. Assure-toi de valider le token.";
        let result = restructure(prompt, "codegen");
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.structured.contains("[R]"));
        assert!(r.structured.contains("[T]"));
    }
}
