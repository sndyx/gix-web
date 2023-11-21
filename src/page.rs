use std::fs;
use std::path::Path;
use actix_web::{get, web, HttpResponse};
use askama::Template;
use comrak::{format_html_with_plugins, parse_document, Arena, Options, Plugins};
use comrak::nodes::{AstNode, NodeCode, NodeValue};
use comrak::plugins::syntect::SyntectAdapter;
use gix::Commit;

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
    let path = Path::new("C:\\Users\\Sandy\\IdeaProjects")
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

    let result = fs::read_to_string(path.join("readme.md")).ok()
        .map(|s| render_markdown_without_title(s));

    let title = if result.is_some() {
        result.clone().unwrap().0
    } else { None };
    let readme = if result.is_some() {
        Some(result.unwrap().1)
    } else { None };

    let r = fs::read_to_string(path.join("readme.md")).ok().map(
        |s| render_markdown(s, |node| {

        })
    );


    let template = IndexTemplate {
        name: "",
        title: title.as_deref(),
        branches: branches.iter().map(|s| &**s).collect(),
        tags: tags.iter().map(|s| &**s).collect(),
        commits: walk,
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

fn render_markdown<'a, F>(markdown: String, f: F)
where
    F: Fn(&'a AstNode<'a>)
{
    let adapter = SyntectAdapter::new("base16-ocean.light");
    let mut options = Options::default();
    let mut plugins = Plugins::default();

    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.header_ids = Some("".to_string());
    plugins.render.codefence_syntax_highlighter = Some(&adapter);


}

fn render_markdown_without_title(markdown: String) -> (Option<String>, String) {
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
        match node.data.clone().into_inner().value {
            NodeValue::Link(mut link) => {
                if !link.url.starts_with("#") && !link.url.starts_with("https://") {
                    link.url = format!("/branch/master/{}", link.url.strip_prefix("/").unwrap_or(&*link.url));
                }
            }
            NodeValue::Heading(heading) => {
                if title.is_none() {
                    if heading.level == 1 {
                        let mut text = String::new();
                        collect_text(node, &mut text);
                        title = Some(text);
                        node.detach();
                    }
                }
            }
            _ => continue,
        }
    }

    let mut html = vec![];
    format_html_with_plugins(root, &options, &mut html, &plugins).unwrap();

    return (
        title,
        String::from_utf8(html).unwrap()
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