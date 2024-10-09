use crate::session::Session;
use crate::{ok, rpc, time};
use catte_tl_schema::*;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub async fn rpc_help_get_config(
    session: Arc<Mutex<Session>>,
    message: rpc::Message<HelpGetConfig>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    let locked_session = session.lock().await;
    ok!(
        message,
        Config {
            date: time!(),
            expires: time!() + 1800,
            test_mode: false,
            this_dc: 2,
            dc_options: vec![
                DcOption {
                    id: 1,
                    ip_address: locked_session.config.host.clone(),
                    port: locked_session.config.actual_port as i32,
                    ipv6: false,
                    media_only: false,
                    tcpo_only: false,
                    cdn: false,
                    is_static: false,
                    this_port_only: true,
                    secret: None,
                },
                DcOption {
                    id: 2,
                    ip_address: locked_session.config.host.clone(),
                    port: locked_session.config.actual_port as i32,
                    ipv6: false,
                    media_only: false,
                    tcpo_only: false,
                    cdn: false,
                    is_static: false,
                    this_port_only: true,
                    secret: None,
                },
                DcOption {
                    id: 3,
                    ip_address: locked_session.config.host.clone(),
                    port: locked_session.config.actual_port as i32,
                    ipv6: false,
                    media_only: false,
                    tcpo_only: false,
                    cdn: false,
                    is_static: false,
                    this_port_only: true,
                    secret: None,
                },
                DcOption {
                    id: 4,
                    ip_address: locked_session.config.host.clone(),
                    port: locked_session.config.actual_port as i32,
                    ipv6: false,
                    media_only: false,
                    tcpo_only: false,
                    cdn: false,
                    is_static: false,
                    this_port_only: true,
                    secret: None,
                },
                DcOption {
                    id: 5,
                    ip_address: locked_session.config.host.clone(),
                    port: locked_session.config.actual_port as i32,
                    ipv6: false,
                    media_only: false,
                    tcpo_only: false,
                    cdn: false,
                    is_static: false,
                    this_port_only: true,
                    secret: None,
                }
            ],
            dc_txt_domain_name: "localhost".into(),
            chat_size_max: 200,
            megagroup_size_max: 200000,
            forwarded_count_max: 100,
            online_update_period_ms: 210000,
            offline_blur_timeout_ms: 5000,
            offline_idle_timeout_ms: 30000,
            online_cloud_timeout_ms: 300000,
            notify_cloud_delay_ms: 30000,
            notify_default_delay_ms: 1500,
            push_chat_period_ms: 60000,
            push_chat_limit: 2,
            edit_time_limit: 172800,
            revoke_time_limit: 2147483647,
            revoke_pm_time_limit: 2147483647,
            rating_e_decay: 2419200,
            stickers_recent_limit: 200,
            channels_read_media_period: 604800,
            call_receive_timeout_ms: 20000,
            call_ring_timeout_ms: 90000,
            call_connect_timeout_ms: 30000,
            call_packet_timeout_ms: 10000,
            me_url_prefix: "http://localhost/".into(),
            caption_length_max: 1024,
            message_length_max: 4096,
            webfile_dc_id: 1,
            default_p2p_contacts: true,
            preload_featured_stickers: false,
            revoke_pm_inbox: true,
            blocked_mode: false,
            force_try_ipv6: false,
            gif_search_username: Some("gif".into()),
            venue_search_username: Some("foursquare".into()),
            img_search_username: Some("bing".into()),
            tmp_sessions: None,
            autoupdate_url_prefix: None,
            static_maps_provider: None,
            suggested_lang_code: None,
            lang_pack_version: None,
            base_lang_pack_version: None,
            reactions_default: None,
            autologin_token: None,
        }
    )
}

pub async fn rpc_help_get_nearest_dc(
    _session: Arc<Mutex<Session>>,
    message: rpc::Message<HelpGetNearestDc>,
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {
    ok!(
        message,
        NearestDc {
            country: "en".to_string(),
            this_dc: 2,
            nearest_dc: 2,
        }
    )
}
