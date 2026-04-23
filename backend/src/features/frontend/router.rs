use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use super::{
    article_category_delete, article_category_list, article_category_save, article_delete,
    article_list, article_save, console_menu_delete, console_menu_list, console_menu_save,
    frontend_nav_delete, frontend_nav_list, frontend_nav_save, frontend_settings_get,
    frontend_settings_save, member_delete, member_list, member_save, public_article_list,
    public_frontend_nav_list, public_frontend_settings_get,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/frontend/nav/list", get(frontend_nav_list))
        .route("/frontend/nav/save", post(frontend_nav_save))
        .route("/frontend/nav/delete", post(frontend_nav_delete))
        .route("/frontend/settings/get", get(frontend_settings_get))
        .route("/frontend/settings/save", post(frontend_settings_save))
        .route(
            "/frontend/article-category/list",
            get(article_category_list),
        )
        .route(
            "/frontend/article-category/save",
            post(article_category_save),
        )
        .route(
            "/frontend/article-category/delete",
            post(article_category_delete),
        )
        .route("/frontend/article/list", get(article_list))
        .route("/frontend/article/save", post(article_save))
        .route("/frontend/article/delete", post(article_delete))
        .route("/frontend/member/list", get(member_list))
        .route("/frontend/member/save", post(member_save))
        .route("/frontend/member/delete", post(member_delete))
        .route("/frontend/console-menu/list", get(console_menu_list))
        .route("/frontend/console-menu/save", post(console_menu_save))
        .route("/frontend/console-menu/delete", post(console_menu_delete))
        .route(
            "/public/frontend/settings",
            get(public_frontend_settings_get),
        )
        .route("/public/frontend/nav", get(public_frontend_nav_list))
        .route("/public/frontend/articles", get(public_article_list))
}
