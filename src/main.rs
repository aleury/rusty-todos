use rusty_todos::Todo;
use serde::Deserialize;
use toasty::Db;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    router::{
        Form, Router, RouterBuilderDiscoverExt, SeeOther, Slot, layout, page, path_param, route,
        see_other,
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

    // An in-memory database keeps the example self-contained; point the URL at
    // a file (e.g. "sqlite:todos.db") to persist todos across restarts.
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

#[page("/")]
async fn home(cx: &Cx) -> Result {
    view! {
        heading(text:"Rusty Todos")
        todo_form()

        let todos = Todo::all()
            .order_by(Todo::fields().id().asc())
            .exec(&mut db(cx))
            .await?;

        <ul class="divide-y divide-slate-100">
            for todo in todos {
                todo_row(todo: &todo)
            }
        </ul>
    }
}

#[component]
async fn heading(text: &str) -> Result {
    view! {
        <h1 class="mb-4 text-xl font-semibold text-slate-900">(text)</h1>
    }
}

#[component]
async fn todo_form() -> Result {
    view! {
        <form method="post" action="/todos" class="mb-4 flex gap-2">
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

#[component]
async fn todo_row(todo: &Todo) -> Result {
    view! {
        <li class="flex items-center gap-3 py-2.5">
            <form method="post" action=(("/todos/", todo.id, "/toggle")) class="flex-1">
                <label class="flex cursor-pointer items-center gap-3">
                    <input
                        type="checkbox"
                        checked=(todo.done)
                        onchange="this.form.submit()"
                        class="h-4 w-4 cursor-pointer rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                    />
                    if todo.done {
                        <s class="text-sm text-slate-400">(&todo.title)</s>
                    } else {
                        <span class="text-sm text-slate-700">(&todo.title)</span>
                    }
                </label>
            </form>
            <form method="post" action=(("/todos/", todo.id, "/delete"))>
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
    Ok(see_other("/"))
}

#[path_param(error = bad_request)]
struct TodoId(u64);

#[route(POST "/todos/{todo_id}/toggle")]
async fn toggle(cx: &Cx) -> Result<SeeOther> {
    let mut db = db(cx);
    let mut todo = Todo::get_by_id(&mut db, *path_param::<TodoId>(cx)?).await?;
    let done = !todo.done;
    toasty::update!(todo { done }).exec(&mut db).await?;
    Ok(see_other("/"))
}

#[route(POST "/todos/{todo_id}/delete")]
async fn delete(cx: &Cx) -> Result<SeeOther> {
    Todo::delete_by_id(&mut db(cx), *path_param::<TodoId>(cx)?).await?;
    Ok(see_other("/"))
}
