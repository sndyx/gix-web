use std::cell::Cell;
use std::path::Path;
use actix_web::{get, web, Result, HttpRequest, HttpResponse};
use askama::Template;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use comrak::nodes::{AstNode, NodeCode, NodeValue};
use comrak::plugins::syntect::SyntectAdapter;
use std::fs;
use gix::Commit;

pub struct RepoData {
    pub(crate) path: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
    title: Option<&'a str>,
    branches: Vec<&'a str>,
    tags: Vec<&'a str>,
    commits: Vec<Commit<'a>>,
    readme: Option<&'a str>
}

#[get("/{repo}")]
pub async fn index(path: web::Path<String>) -> HttpResponse {
    let path = Path::new("/Users/25alexandercapitos/sndy/Documents/")
        .join(path.into_inner());
    let repo = match gix::open(&path) {
        Ok(repo) => repo,
        Err(error) => return HttpResponse::NotFound().body(error.to_string())
    };

    let references = repo.references().unwrap();
    let branches: Vec<String> = references.local_branches().unwrap()
        .map(|b| b.unwrap().inner.name.shorten().to_string())
        .collect();
    let tags: Vec<String> = references.tags().unwrap()
        .map(|t| t.unwrap().inner.name.shorten().to_string())
        .collect();

    let head_commit = repo.head_commit().unwrap();
    let walk = head_commit.ancestors().all().unwrap().take(5).map(|info| info.unwrap().object().unwrap()).collect();


    let (title, readme) = fs::read_to_string(path.join("readme.md")).ok()
        .map(|s| render_markdown_without_title(s));

    let template = IndexTemplate {
        name: "",
        title: readme.0,
        branches: branches.iter().map(|s| &**s).collect(),
        tags: tags.iter().map(|s| &**s).collect(),
        commits: walk,
        readme
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

fn render_markdown_without_title<'a>(markdown: String) -> (Option<&'a str>, &'a str) {
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

    let mut title: Option<String> = None;
    for node in root.children() {
        if let NodeValue::Link(ref mut link) = node.data.borrow_mut().value {
            if !link.url.starts_with("#") && !link.url.starts_with("https://") {
                link.url = format!("/branch/master/{}", link.url.strip_prefix("/").unwrap_or(&*link.url));
            }
        }
        else if title.is_none() {
            if let NodeValue::Heading(ref mut heading) = node.data.borrow_mut().value {
                if heading.level == 1 {
                    let mut text = String::new();
                    collect_text(node, &mut text);
                    title = Some(text);
                    node.detach();
                }
            }
        }
    }

    let mut html = vec![];
    format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();

    return (
        title.as_deref(),
        String::from_utf8(html).unwrap().as_str()
    )
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut String) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.push_str(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}