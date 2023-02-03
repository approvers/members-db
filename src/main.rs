#![deny(
    clippy::allow_attributes_without_reason,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::exit,
    clippy::filetype_is_file,
    clippy::float_arithmetic,
    clippy::float_cmp_const,
    clippy::format_push_string,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::let_underscore_must_use,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::mod_module_files,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_outside_block,
    clippy::separated_literal_suffix,
    clippy::string_to_string,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::verbose_file_reads
)]
#![warn(
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use dotenv::dotenv;

use crate::controller::discord::start_discord_bot;
use crate::usecase::firebase::get_firebase_usecases;
// use crate::controller::http::start_http_server;

pub(crate) mod controller;
pub(crate) mod infra;
pub(crate) mod model;
pub(crate) mod usecase;
pub(crate) mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // tokio::try_join!(start_http_server(), start_discord_bot(),).map(|_| ())
    start_discord_bot(get_firebase_usecases().await?).await
}
