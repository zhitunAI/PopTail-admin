use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use super::{
    authority_btn_get, authority_btn_set, authority_list, authority_set_role_users,
    authority_upsert, authority_users, menu_authority_get, menu_authority_set, menu_list,
    menu_roles_get, menu_roles_set, menu_tree, menu_upsert, user_list, user_policy_paths,
    user_policy_paths_set,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/casbin/getPolicyPathByAuthorityId",
            post(user_policy_paths),
        )
        .route(
            "/casbin/setPolicyPathByAuthorityId",
            post(user_policy_paths_set),
        )
        .route("/user/getUserList", post(user_list))
        .route("/authority/getAuthorityList", post(authority_list))
        .route("/authority/saveAuthority", post(authority_upsert))
        .route("/authority/getUsersByAuthority", get(authority_users))
        .route("/authority/setRoleUsers", post(authority_set_role_users))
        .route("/authorityBtn/getAuthorityBtn", post(authority_btn_get))
        .route("/authorityBtn/setAuthorityBtn", post(authority_btn_set))
        .route("/menu/getMenu", post(menu_tree))
        .route("/menu/getMenuList", post(menu_list))
        .route("/menu/saveMenu", post(menu_upsert))
        .route("/menu/getMenuAuthority", post(menu_authority_get))
        .route("/menu/addMenuAuthority", post(menu_authority_set))
        .route("/menu/getMenuRoles", get(menu_roles_get))
        .route("/menu/setMenuRoles", post(menu_roles_set))
}
