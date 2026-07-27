use rusty_todos::Todo;
use serde::Deserialize;
use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    router::{
        Form, Router, RouterBuilderDiscoverExt, SeeOther, Slot, layout, page, path_param,
        query_params, route, see_other,
    },
    tailwind,
    view::{component, view},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Toasty does not create tables on connect, so the schema has to be in
    // place already: run `just migrate` before starting the server.
    let db = Db::builder()
        .models(toasty::models!(Todo))
        .connect("sqlite:./todos.db")
        .await
        .unwrap();

    let router = Router::builder()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .app_context(db)
        .build();

    topcoat::start(router).await.unwrap();
}

// Toasty statements borrow the handle mutably, so each handler clones the
// shared `Db` (a cheap handle to the underlying connection pool) out of app
// context.
fn db(cx: &Cx) -> Db {
    app_context::<Db>(cx).clone()
}

#[layout("/")]
async fn root(slot: Slot<'_>) -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Toasty Todos"</title>
                topcoat::dev::script()
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body class="min-h-screen bg-slate-50 flex items-start justify-center p-6">
                <div class="w-full max-w-md mt-12 rounded-2xl bg-white p-6 shadow-sm ring-1 ring-slate-200">
                    (slot.await?)
                </div>
            </body>
        </html>
    }
}

#[derive(Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Filter {
    Active,
    Done,
}

// Every field is optional, so an unrecognized value can safely reload the page
// without a query string rather than answering a browsing user with a 400.
#[query_params(error = redirect("?"))]
struct TodoQuery {
    filter: Option<Filter>,
}

// The POST routes carry no query string of their own, so the active filter
// rides along on each form's action and is read back here to redirect the user
// to the view they were looking at.
fn query_suffix(filter: Option<Filter>) -> &'static str {
    match filter {
        Some(Filter::Active) => "?filter=active",
        Some(Filter::Done) => "?filter=done",
        None => "",
    }
}

fn home_path(filter: Option<Filter>) -> &'static str {
    match filter {
        Some(Filter::Active) => "/?filter=active",
        Some(Filter::Done) => "/?filter=done",
        None => "/",
    }
}

// Redirecting in response to a form submission would be surprising, so the POST
// handlers treat an unparseable filter as no filter instead of using `?`.
fn current_filter(cx: &Cx) -> Option<Filter> {
    query_params::<TodoQuery>(cx).ok().and_then(|q| q.filter)
}

#[page("/")]
async fn home(cx: &Cx) -> Result {
    let filter = query_params::<TodoQuery>(cx)?.filter;

    let mut query = Todo::all();
    if let Some(filter) = filter {
        query = query.filter(Todo::fields().done().eq(filter == Filter::Done));
    }
    let todos = query
        .order_by(Todo::fields().created_at().asc())
        .exec(&mut db(cx))
        .await?;

    view! {
        heading(text:"Rusty Todos")
        todo_form(filter: filter)
        filter_links(current: filter)

        if todos.is_empty() {
            <p class="py-2.5 text-sm text-slate-400">"Nothing here."</p>
        } else {
            <ul class="divide-y divide-slate-100">
                for todo in todos {
                    todo_row(todo: &todo, filter: filter)
                }
            </ul>
        }
    }
}

#[component]
async fn filter_links(current: Option<Filter>) -> Result {
    view! {
        <nav class="mb-4 flex gap-3 text-sm">
            filter_link(href: "/", label: "All", selected: current.is_none())
            filter_link(href: "/?filter=active", label: "Active", selected: current == Some(Filter::Active))
            filter_link(href: "/?filter=done", label: "Done", selected: current == Some(Filter::Done))
        </nav>
    }
}

#[component]
async fn filter_link(href: &str, label: &str, selected: bool) -> Result {
    view! {
        if selected {
            <a href=(href) class="font-medium text-indigo-600">(label)</a>
        } else {
            <a href=(href) class="text-slate-400 hover:text-slate-600">(label)</a>
        }
    }
}

#[component]
async fn heading(text: &str) -> Result {
    view! {
        <h1 class="mb-4 text-xl font-semibold text-slate-900">(text)</h1>
    }
}

#[component]
async fn todo_form(filter: Option<Filter>) -> Result {
    view! {
        <form method="post" action=(("/todos", query_suffix(filter))) class="mb-4 flex gap-2">
            <input
                type="text"
                name="title"
                placeholder="What needs doing?"
                required=(true)
                class="flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-900 placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
            <button
                type="submit"
                class="cursor-pointer rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700"
            >"Add"</button>
        </form>
    }
}

// The full timestamp shown on hover, e.g. "Jul 27, 2026 at 12:08 AM".
const HOVER_FORMAT: &str = "%b %-d, %Y at %-I:%M %p";

// Coarse "time ago" wording, falling back to a bare date after a week. Every
// action reloads the page, so a server-rendered relative time never goes stale.
fn relative(ts: jiff::Timestamp) -> String {
    let secs = (jiff::Timestamp::now().as_second() - ts.as_second()).max(0);
    match secs {
        s if s < 60 => "just now".to_string(),
        s if s < 60 * 60 => format!("{}m ago", s / 60),
        s if s < 60 * 60 * 24 => format!("{}h ago", s / (60 * 60)),
        s if s < 60 * 60 * 24 * 7 => format!("{}d ago", s / (60 * 60 * 24)),
        _ => zoned(ts, "%b %-d"),
    }
}

fn zoned(ts: jiff::Timestamp, fmt: &str) -> String {
    ts.to_zoned(jiff::tz::TimeZone::system())
        .strftime(fmt)
        .to_string()
}

#[component]
async fn todo_row(todo: &Todo, filter: Option<Filter>) -> Result {
    // `created_at` and `updated_at` each evaluate their own `now()` at insert,
    // so they land microseconds apart on a brand new todo. Only treat a whole
    // second of drift as a real edit.
    let edited = todo.updated_at.as_second() - todo.created_at.as_second() >= 1;

    view! {
        <li class="flex items-start gap-3 py-2.5">
            <form method="post" action=(("/todos/", todo.id, "/toggle", query_suffix(filter))) class="flex-1">
                <label class="flex cursor-pointer items-start gap-3">
                    <input
                        type="checkbox"
                        checked=(todo.done)
                        onchange="this.form.submit()"
                        class="mt-0.5 h-4 w-4 cursor-pointer rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                    />
                    <span class="flex flex-col gap-0.5">
                        if todo.done {
                            <s class="text-sm text-slate-400">(&todo.title)</s>
                        } else {
                            <span class="text-sm text-slate-700">(&todo.title)</span>
                        }
                        <span class="text-xs text-slate-400">
                            <span title=(zoned(todo.created_at, HOVER_FORMAT))>
                                (relative(todo.created_at))
                            </span>
                            if edited {
                                " · edited "
                                <span title=(zoned(todo.updated_at, HOVER_FORMAT))>
                                    (relative(todo.updated_at))
                                </span>
                            }
                        </span>
                    </span>
                </label>
            </form>
            <form method="post" action=(("/todos/", todo.id, "/delete", query_suffix(filter)))>
                <button type="submit" class="cursor-pointer text-sm text-slate-400 hover:text-red-600">"Delete"</button>
            </form>
        </li>
    }
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

#[route(POST "/todos")]
async fn create(cx: &Cx, Form(new_todo): Form<NewTodo>) -> Result<SeeOther> {
    let title = new_todo.title.trim();
    if !title.is_empty() {
        toasty::create!(Todo { title, done: false })
            .exec(&mut db(cx))
            .await?;
    }
    Ok(see_other(home_path(current_filter(cx))))
}

#[path_param(error = bad_request)]
struct TodoId(u64);

#[route(POST "/todos/{todo_id}/toggle")]
async fn toggle(cx: &Cx) -> Result<SeeOther> {
    let mut db = db(cx);
    let mut todo = Todo::get_by_id(&mut db, *path_param::<TodoId>(cx)?).await?;
    let done = !todo.done;
    toasty::update!(todo { done }).exec(&mut db).await?;
    Ok(see_other(home_path(current_filter(cx))))
}

#[route(POST "/todos/{todo_id}/delete")]
async fn delete(cx: &Cx) -> Result<SeeOther> {
    Todo::delete_by_id(&mut db(cx), *path_param::<TodoId>(cx)?).await?;
    Ok(see_other(home_path(current_filter(cx))))
}
