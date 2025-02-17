use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, get_local_user_view_from_jwt, is_admin, site::*};
use lemmy_db_queries::{source::site::Site_, Crud};
use lemmy_db_schema::source::site::{Site, *};
use lemmy_db_views::site_view::SiteView;
use lemmy_utils::{
  utils::{check_slurs, check_slurs_opt},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateSite {
  type Response = SiteResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<SiteResponse, LemmyError> {
    let data: &CreateSite = &self;

    let read_site = move |conn: &'_ _| Site::read_simple(conn);
    if blocking(context.pool(), read_site).await?.is_ok() {
      return Err(ApiError::err("site_already_exists").into());
    };

    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs_opt(&data.description)?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let site_form = SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      icon: Some(data.icon.to_owned().map(|url| url.into())),
      banner: Some(data.banner.to_owned().map(|url| url.into())),
      creator_id: local_user_view.person.id,
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
      updated: None,
    };

    let create_site = move |conn: &'_ _| Site::create(conn, &site_form);
    if blocking(context.pool(), create_site).await?.is_err() {
      return Err(ApiError::err("site_already_exists").into());
    }

    let site_view = blocking(context.pool(), move |conn| SiteView::read(conn)).await??;

    Ok(SiteResponse { site_view })
  }
}
