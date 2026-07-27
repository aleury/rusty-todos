use std::sync::{Mutex, atomic::AtomicU64};

use serde::Deserialize;
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

#[tokio::main]
async fn main() {
    let todos = Todos {
        items: Mutex::new(vec![
            Todo {
                id: 1,
                text: "Learn Rust".to_string(),
                done: false,
            },
            Todo {
                id: 2,
                text: "Learn Topcoat".to_string(),
                done: false,
            },
            Todo {
                id: 3,
                text: "Build a todo app with Topcoat & Rust".to_string(),
                done: false,
            },
        ]),
        next_id: AtomicU64::new(4),
    };

    let router = Router::builder()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .app_context(todos)
        .build();

    topcoat::start(router).await.unwrap();
}

#[derive(Debug, Clone)]
struct Todo {
    id: u64,
    text: String,
    done: bool,
}

#[derive(Debug, Default)]
struct Todos {
    items: Mutex<Vec<Todo>>,
    next_id: AtomicU64,
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

fn get_todos(cx: &Cx) -> Vec<Todo> {
    app_context::<Todos>(cx).items.lock().unwrap().clone()
}

#[page("/")]
async fn home(cx: &Cx) -> Result {
    view! {
        heading(text:"Rusty Todos")
        todo_form()
        <ul class="divide-y divide-slate-100">
            for item in get_todos(cx) {
                todo_row(todo: &item)
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
                        <s class="text-sm text-slate-400">(&todo.text)</s>
                    } else {
                        <span class="text-sm text-slate-700">(&todo.text)</span>
                    }
                </label>
            </form>
            <form method="post" action=(("/todos/", todo.id, "/delete"))>
                <button type="submit" class="cursor-pointer text-sm text-slate-400 hover:text-red-600">"Delete"</button>
            </form>
        </li>
    }
}

#[path_param(error = bad_request)]
struct TodoId(u64);

#[route(POST "/todos/{todo_id}/toggle")]
async fn toggle(cx: &Cx) -> Result<SeeOther> {
    let todos = app_context::<Todos>(cx);
    let todo_id = *path_param::<TodoId>(cx)?;
    let mut items = todos.items.lock().expect("todo items");
    if let Some(item) = items.iter_mut().find(|i| i.id == todo_id) {
        item.done = !item.done;
    }
    Ok(see_other("/"))
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
}

#[route(POST "/todos")]
async fn create(cx: &Cx, Form(new_todo): Form<NewTodo>) -> Result<SeeOther> {
    let title = new_todo.title.trim();
    if !title.is_empty() {
        let todos = app_context::<Todos>(cx);
        let mut items = todos.items.lock().expect("todo items");
        items.push(Todo {
            id: todos
                .next_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            text: title.to_string(),
            done: false,
        });
    }
    Ok(see_other("/"))
}

#[route(POST "/todos/{todo_id}/delete")]
async fn delete(cx: &Cx) -> Result<SeeOther> {
    let todo_id = *path_param::<TodoId>(cx)?;
    let todos = app_context::<Todos>(cx);
    let mut items = todos.items.lock().expect("todo items");
    items.retain(|i| i.id != todo_id);
    Ok(see_other("/"))
}
