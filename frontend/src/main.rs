use models::Category;
use reqwasm::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

async fn get_categories() -> Vec<Category> {
    return Request::get("/categories")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/categories")]
    CategoriesView,
}

#[function_component(TaskView)]
fn task_view() -> Html {
    let categories = use_state(|| None);
    {
        let categories = categories.clone();
        use_effect_with_deps(
            move |_| {
                let categories = categories.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_categories = get_categories().await;
                    categories.set(Some(fetched_categories));
                });
                || ()
            },
            (),
        );
    }
    if let Some(categories) = &*categories {
        let mut categories_html = categories
            .into_iter()
            .map(|category| {
                html! {
                    <div class="row">
                    <div class="col">
                    {category.name.clone()}
                    </div>
                    </div>
                }
            })
            .collect::<Html>();
        return categories_html;
    } else {
        return html! { <div>{"Loading..."}</div> };
    }
}

struct CategoriesList;

impl Component for CategoriesList {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let categories = use_state(|| None);
        {
            let categories = categories.clone();
            use_effect_with_deps(
                move |_| {
                    let categories = categories.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let fetched_categories = get_categories().await;
                        categories.set(Some(fetched_categories));
                    });
                    || ()
                },
                (),
            );
        }
        if let Some(categories) = &*categories {
            let mut categories_html = categories
                .into_iter()
                .map(|category| {
                    html! {
                        <div class="row">
                        <div class="col">
                        {"Some string"}
                        </div>
                        </div>
                    }
                })
                .collect::<Html>();
            return categories_html;
        } else {
            return html! { <div>{"Loading..."}</div> };
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes.clone() {
        Route::CategoriesView => {
            html! { <CategoriesList /> }
        }
    }
}

fn main() {
    yew::start_app::<TaskView>();
}
