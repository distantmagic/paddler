use crate::web_admin_panel_service::dashboard_assets::DashboardAssets;
use crate::web_admin_panel_service::template_data::TemplateData;

pub struct AppData {
    pub dashboard_assets: DashboardAssets,
    pub template_data: TemplateData,
}
