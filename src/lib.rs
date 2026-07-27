#[derive(Debug, toasty::Model)]
pub struct Todo {
    #[key]
    #[auto]
    pub id: u64,
    pub title: String,
    pub done: bool,
}
