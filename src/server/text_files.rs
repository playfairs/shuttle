use std::path::Path;

const TEXT_EXTENSIONS: &[&str] = &[
    "rs", "js", "ts", "jsx", "tsx", "py", "rb", "go", "java", "c", "cpp", "h", "hpp", "cs", "php",
    "html", "htm", "css", "scss", "sass", "less", "json", "xml", "yaml", "yml", "toml", "md",
    "markdown", "rst", "adoc", "tex", "bib", "txt", "sh", "bash", "zsh", "fish", "csh", "tcsh",
    "sql", "r", "swift", "kt", "kts", "dart", "lua", "pl", "pm", "t", "scala", "sc", "hs", "lhs",
    "clj", "cljs", "cljc", "edn", "fs", "fsi", "fsx", "fsscript", "v", "nim", "nims", "cr", "ex",
    "exs", "elm", "purs", "ml", "mli", "re", "rei", "lisp", "lsp", "scm", "rkt", "rash", "coffee",
    "litcoffee", "tsv", "csv", "ini", "cfg", "conf", "log", "dockerfile", "containerfile", "makefile",
    "mk", "mak", "cmake", "gradle", "properties", "manifest", "lock", "mod", "sum", "build", "nix",
    "hsm", "env", "envrc", "editorconfig", "eslintrc", "prettierrc", "babelrc", "tsconfig", "vimrc",
    "gvimrc", "nvimrc", "emacs", "el", "spacemacs", "tmux", "screenrc", "inputrc", "bashrc",
    "bash_profile", "zshrc", "zprofile", "profile", "gitignore", "gitattributes", "hgignore",
    "cvsignore", "npmrc", "yarnrc", "pnpmrc", "bower", "gemfile", "pipfile", "proto", "thrift",
    "avsc", "graphql", "gql", "travis", "circleci", "jenkinsfile", "azure", "bitbucket", "gitlab",
    "justfile", "taskfile", "rake", "procfile", "heroku", "vercel", "netlify", "now", "wrangler",
    "faas", "serverless", "sam", "terraform", "hcl", "tf", "tfvars", "vault", "nomad", "consul",
    "packer", "cdk", "cf", "cfn", "arm", "bicep", "k8s", "dotenv", "ps1", "psm1", "psd1", "bat",
    "cmd", "vbs", "wsf", "wsc", "reg", "inf", "au3", "ahk", "iss", "nsi", "nsh",
];

const TEXT_FILENAMES: &[&str] = &[
    "makefile", "dockerfile", "containerfile", "justfile", "procfile", "gemfile", "rakefile",
];

pub fn is_text_file(path: &Path) -> bool {
    let extension = path.extension().and_then(|e| e.to_str());
    
    if let Some(ext) = extension {
        TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
    } else {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        TEXT_FILENAMES.contains(&filename.to_lowercase().as_str())
    }
}
