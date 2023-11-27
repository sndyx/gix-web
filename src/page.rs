use std::fs;
use std::path::Path;
use actix_web::{get, web, HttpResponse};
use askama::Template;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use comrak::nodes::NodeValue;
use comrak::plugins::syntect::SyntectAdapter;
use gix::Commit;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
    branches: Vec<&'a str>,
    tags: Vec<&'a str>,
    commits: Vec<Commit<'a>>,
    readme: Option<&'a str>
}

#[get("/{repo}")]
pub async fn index(path: web::Path<String>) -> HttpResponse {
    let path = Path::new("/Users/25alexandercapitos/sndy/Documents")
        .join(path.into_inner());
    let repo = match gix::open(&path) {
        Ok(repo) => repo,
        Err(error) => return HttpResponse::NotFound().body(error.to_string())
    };
    let name = path.file_name().unwrap().to_str().unwrap();

    let references = repo.references().unwrap();
    let branches: Vec<String> = references.local_branches().unwrap()
        .map(|it| it.unwrap().inner.name.shorten().to_string())
        .collect();
    let tags: Vec<String> = references.tags().unwrap()
        .map(|it| it.unwrap().inner.name.shorten().to_string())
        .collect();

    let head_commit = repo.head_commit().unwrap();
    let commits = head_commit.ancestors().all().unwrap().take(5).map(|info| info.unwrap().object().unwrap()).collect();

    let readme = fs::read_to_string(path.join("readme.md")).ok()
        .map(|s| render_markdown(s));

    let template = IndexTemplate {
        name,
        branches: branches.iter().map(|s| &**s).collect(),
        tags: tags.iter().map(|s| &**s).collect(),
        commits,
        readme: readme.as_deref(),
    };

    render_template(template)
}

#[derive(Template)]
#[template(path = "head.html")]
struct HeadTemplate<'a> {
    name: &'a str,
    branch: &'a str,
}

#[get("/{repo}/refs/head/{branch}")]
pub async fn head(path: web::Path<(String, String)>) -> HttpResponse {
    let path = Path::new("/Users/25alexandercapitos/sndy/Documents")
        .join(path.into_inner().0);
    let repo = match gix::open(&path) {
        Ok(repo) => repo,
        Err(error) => return HttpResponse::NotFound().body(error.to_string())
    };
    let name = path.file_name().unwrap().to_str().unwrap();

    let references = repo.references().unwrap();
    let branches: Vec<String> = references.local_branches().unwrap()
        .map(|it| it.unwrap().inner.name.shorten().to_string())
        .collect();
    let tags: Vec<String> = references.tags().unwrap()
        .map(|it| it.unwrap().inner.name.shorten().to_string())
        .collect();

    let head_commit = repo.head_commit().unwrap();
    let commits = head_commit.ancestors().all().unwrap().take(5).map(|info| info.unwrap().object().unwrap()).collect();

    let readme = fs::read_to_string(path.join("readme.md")).ok()
        .map(|s| render_markdown(s));

    let template = IndexTemplate {
        name,
        branches: branches.iter().map(|s| &**s).collect(),
        tags: tags.iter().map(|s| &**s).collect(),
        commits,
        readme: readme.as_deref(),
    };

    render_template(template)
}

fn render_template(template: impl Template) -> HttpResponse {
    let page_content = match template.render() {
        Ok(contents) => contents,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string())
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(page_content)
}

fn render_markdown(markdown: String) -> String {
    let adapter = SyntectAdapter::new("base16-ocean.light");
    let mut options = Options::default();
    let mut plugins = Plugins::default();

    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Some("".to_string());
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    let arena = Arena::new();
    let root = parse_document(&arena, markdown.as_str(), &options);

    for node in root.children() {
        match node.data.clone().into_inner().value {
            NodeValue::Link(mut link) => {
                if !link.url.starts_with("#") && !link.url.starts_with("https://") {
                    link.url = format!("/branch/master/{}", link.url.strip_prefix("/").unwrap_or(&*link.url));
                }
            }
            _ => continue,
        }
    }

    let mut html = vec![];
    format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();

    String::from_utf8(html).unwrap()
}