use regex::Regex;

const BASE_URL: &str = "https://paste.mozilla.org/";
const SUPPORTED_LANG: [&str; 63] = ["_text", "_markdown", "_rst", "_code", "applescript", "arduino", "bash", "bat", "c", "clojure", "cmake", "coffee-script", "common-lisp", "console", "cpp", "csharp", "css", "cuda", "dart", "delphi", "diff", "django", "dker", "elixir", "erlang", "go", "handlebars", "haskell", "html", "html+django", "ini", "ipythonconsole", "irc", "java", "js", "json", "jsx", "kotlin", "less", "lua", "make", "matlab", "nginx", "numpy", "objective-c", "perl", "php", "postgresql", "python", "rb", "rst", "rust", "sass", "scss", "sol", "sql", "swift", "tex", "typoscript", "vim", "xml", "xslt", "yaml"];
const SUPPORTED_EXPIRE: [&str; 5] = ["once", "1h", "1d", "1w", "21d"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    match &args[..] {
        [_, ref file, ref time, ref lang] => {
            if !SUPPORTED_LANG.contains(&lang.as_str()) {
                println!("Unsupported language: {}", lang);
                println!("Supported languages: {:?}", SUPPORTED_LANG);
                return Ok(());
            }
            upload_file(file, time, Some(lang.clone())).await?;
        },
        [_, ref file,ref time] => {
            upload_file(file, time, None).await?;
        },
        [_, ref file] => {
            upload_file(file, "once", None).await?;
        },
        _ => {
            println!("Usage: paste <file> [time: once(default), 1h, 1d, 1w, 21d] [lang]");
            println!("Supported languages: {:?}", SUPPORTED_LANG);
        }
    }
    Ok(())
}

async fn upload_file(file: &str, time: &str, lang: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if !SUPPORTED_EXPIRE.contains(&time) {
        println!("Unsupported expire time: {}", time);
        println!("Supported expire time: {:?}", SUPPORTED_EXPIRE);
        return Ok(());
    }

    let path = std::path::Path::new(file);
    let file_content = std::fs::read_to_string(file)?;
    let lang = lang.or(
        path.file_name()
        .and_then(|file| file.to_str())
        .and_then(|file| map_filename_to_lang(file))
    ).unwrap_or("_code".to_string());

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(1024))
        .build()?;

    let res = client.get(BASE_URL)
        .send()
        .await?;

    let html = res.text().await?;
    let document = dom_query::Document::from(html);

    let token = document.select("input[name=csrfmiddlewaretoken]").attr("value").unwrap().to_string();

    let mut form = std::collections::HashMap::new();
    form.insert("csrfmiddlewaretoken", token);
    form.insert("content", file_content);
    form.insert("expires", match time {
        "once" => "onetime",
        "1h" => "3600",
        "1d" => "86400",
        "1w" => "604800",
        "21d" => "2073600",
        _ => "onetime",
    }.to_string());
    form.insert("lexer", lang);
    form.insert("title", "".to_string());


    let res = client.post(BASE_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Referer", BASE_URL)
        .header("Origin", BASE_URL)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36")
        .form(&form)
        .send()
        .await?;

    println!("Paste url: {}", res.url().to_string());

    Ok(())
}


fn map_filename_to_lang(file: &str) -> Option<String> {
    // Convert the file name to lowercase for case-insensitive matching
    let file_lower = file.to_lowercase();

    // Handle special cases that don't follow the regular file extension pattern
    let special_cases = match file_lower.as_str() {
        "dockerfile" => Some("docker"),
        "makefile" => Some("make"),
        "cmakelists.txt" => Some("cmake"),
        "nginx.conf" => Some("nginx"),
        f if f.contains("nginx") => Some("nginx"),
        _ => None,
    };

    if special_cases.is_some() {
        return special_cases.map(|l| l.to_string());
    }

    // Create a regex to extract the file extension for standard cases
    let re = Regex::new(r"\.([a-zA-Z0-9+_-]+)$").unwrap();

    // Check if the file matches the regex and capture the extension
    if let Some(caps) = re.captures(&file_lower) {
        if let Some(ext) = caps.get(1) {
            let ext = ext.as_str();
            // Map file extension to programming languages
            let lang = match ext {
                "txt" => Some("_text"),
                "md" => Some("_markdown"),
                "rst" => Some("_rst"),
                "sh" => Some("bash"),
                "bat" => Some("bat"),
                "c" => Some("c"),
                "lisp" | "lsp" | "cl" => Some("common-lisp"),
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "inc" | "hh" | "h" => Some("cpp"),
                "cs" => Some("csharp"),
                "cmake" | "in" => Some("cmake"),
                "css" => Some("css"),
                "dart" => Some("dart"),
                "patch" | "diff" => Some("diff"),
                "elixir" | "ex" | "exs" => Some("elixir"),
                "erl" => Some("erlang"),
                "go" => Some("go"),
                "hbs" => Some("handlebars"),
                "hs" => Some("haskell"),
                "html" | "htm" | "shtm" | "shtml" => Some("html"),
                "ini" => Some("ini"),
                "java" => Some("java"),
                "js" | "ts" => Some("js"),
                "json" | "jsonl" => Some("json"),
                "tsx" | "jsx" => Some("jsx"),
                "kt" | "kts" => Some("kotlin"),
                "lua" => Some("lua"),
                "m" | "mm" => Some("objective-c"),
                "pl" => Some("perl"),
                "php" => Some("php"),
                "py" => Some("python"),
                "rb" => Some("rb"),
                "rs" => Some("rust"),
                "sass" => Some("sass"),
                "scss" => Some("scss"),
                "sol" => Some("sol"),
                "sql" => Some("sql"),
                "swift" => Some("swift"),
                "tex" => Some("tex"),
                "typoscript" => Some("typoscript"),
                "vim" => Some("vim"),
                "xml" => Some("xml"),
                "xsl" | "xslt" => Some("xslt"),
                "yml" | "yaml" => Some("yaml"),
                _ => None,
            };
            return lang.map(|l| l.to_string());
        }
    }

    None
}