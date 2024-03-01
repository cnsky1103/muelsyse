use anyhow::Result;

mod editor;

fn main() -> Result<()> {
    editor::Editor::new().run()
}
