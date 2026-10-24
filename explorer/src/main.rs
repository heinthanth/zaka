use zakac_explorer::app::app;
use zakac_explorer::state::lang::load_stored_lang;

fn main()
{
    console_error_panic_hook::set_once();
    zakac_i18n::init_locale();
    load_stored_lang();
    leptos::mount::mount_to_body(app);
}
