use std::sync;

use crate::notes::Provide as _;

mod defaults;
mod notes;
mod settings;
mod website;

#[tokio::main]
#[tracing::instrument(name = "main")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let settings = sync::Arc::new(settings::Provider::new());
    let notes = notes::Provider::from(settings.clone());

    let Ok(notes) = notes.sync().await else {
        tracing::error!("Failed to sync notes");
        return;
    };

    dbg!(notes.notes());
}

//fn main() -> Result<()> {
//    print!(
//        r#"
//       .~@`,
//      (__,  \
//          \' \
//           \  \
//            \  \
//             \  `._            __.__
//              \    ~-._  _.==~~     ~~--.._
//               \        '                  ~-.
//                \      _-   -_                `.
//                 \    /       )        .-    .  \
//                  `. |      /  )      (       ;  \
//                    `|     /  /       (       :   '\
//                     \    |  /        |      /       \
//                      |     /`-.______\.     |~-.      \
//                      |   |/           (     |   `.      \_
//                      |   ||            ~\   \      '._    `-.._____..----..___
//                      |   |/             _\   \         ~-.__________.-~~~~~~~~~'''
//      post_notes    .o'___/            .o______)
//
//
//        "#
//    );
//
//    colog::init();
//
//    log::info!("=== Loading Settings ===");
//    let settings = get_settings();
//
//    println!();
//
//    log::info!(
//        "=== Starting to load content from {}. ===",
//        &settings.path.input.display()
//    );
//    let raw = fetch::notes(&settings.path.input).context("Failed to read content")?;
//    let notes = map::notes(raw);
//
//    println!();
//
//    log::info!(
//        "=== Starting to generate content map with {} entrie(s). ===",
//        notes.len()
//    );
//    let content = map::content(&notes);
//
//    println!();
//
//    log::info!("=== Starting to generate navigation. ===");
//    let nav = map::navigation(&notes);
//
//    println!();
//
//    log::info!("=== Starting to build website. ===");
//    build::website(&notes, content, nav, &settings).context("Failed to build website")?;
//
//    Ok(())
//}
